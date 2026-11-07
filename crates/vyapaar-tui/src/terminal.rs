//! Interactive terminal event loop and LLM stream orchestration.

use std::io::{self, Stdout};
use std::sync::Arc;
use std::time::Duration;

use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use rust_decimal::Decimal;
use tokio::sync::mpsc;

use vyapaar_agent_llm::{ChatMessage, ChatRequest, ChatStreamEvent, LlmProvider};

use crate::app::TuiApp;
use crate::ui;

/// Update message emitted by background agent token streamer.
#[derive(Debug)]
pub enum AgentStreamUpdate {
    /// Token text fragment.
    Token(String),
    /// Turn completed normally.
    Finished,
    /// An error occurred during generation.
    Error(String),
}

/// Run the interactive Ratatui terminal workstation.
///
/// Handles terminal setup/teardown, event loop polling, and live token
/// streaming between the in-process LLM provider and the AgentChat tab.
pub async fn run_terminal(
    mut app: TuiApp,
    provider: Arc<dyn LlmProvider>,
) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_loop(&mut terminal, &mut app, provider).await;

    // Clean teardown
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    res
}

async fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut TuiApp,
    provider: Arc<dyn LlmProvider>,
) -> Result<(), io::Error> {
    let (token_tx, mut token_rx) = mpsc::channel::<AgentStreamUpdate>(128);

    loop {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if app.should_quit {
            break;
        }

        // Drain any incoming tokens from background streaming task
        while let Ok(update) = token_rx.try_recv() {
            match update {
                AgentStreamUpdate::Token(delta) => {
                    if let Some(last) = app.chat.last_mut() {
                        last.text.push_str(&delta);
                    }
                }
                AgentStreamUpdate::Finished => {
                    app.is_loading = false;
                }
                AgentStreamUpdate::Error(err) => {
                    if let Some(last) = app.chat.last_mut() {
                        if last.text.is_empty() {
                            last.text = format!("*Error: {err}*");
                        } else {
                            last.text.push_str(&format!("\n\n*Error: {err}*"));
                        }
                    }
                    app.is_loading = false;
                }
            }
        }

        // Check if user submitted a prompt on the Agent tab
        if let Some(prompt) = app.pending_prompt.take() {
            let provider_cloned = Arc::clone(&provider);
            let tx = token_tx.clone();

            tokio::spawn(async move {
                let req = ChatRequest {
                    messages: vec![ChatMessage::user(prompt)],
                    system_prompt: Some("You are Vyapaar, an intelligent quantitative finance and algorithmic trading assistant for Indian equities (NSE/BSE).".to_string()),
                    tools: Vec::new(),
                    temperature: Some(Decimal::new(7, 1)),
                    max_tokens: Some(1024),
                };

                match provider_cloned.chat_stream(&req).await {
                    Ok(mut stream) => {
                        use futures::StreamExt;
                        while let Some(event_res) = stream.next().await {
                            match event_res {
                                Ok(ChatStreamEvent::TextDelta { text }) => {
                                    let _ = tx.send(AgentStreamUpdate::Token(text)).await;
                                }
                                Ok(ChatStreamEvent::MessageEnd { .. }) => {
                                    let _ = tx.send(AgentStreamUpdate::Finished).await;
                                    return;
                                }
                                Err(e) => {
                                    let _ = tx.send(AgentStreamUpdate::Error(e.to_string())).await;
                                    return;
                                }
                                _ => {}
                            }
                        }
                        let _ = tx.send(AgentStreamUpdate::Finished).await;
                    }
                    Err(e) => {
                        let _ = tx.send(AgentStreamUpdate::Error(e.to_string())).await;
                    }
                }
            });
        }

        // Poll for input with short timeout for responsive token rendering
        if event::poll(Duration::from_millis(30))? {
            if let Event::Key(key) = event::read()? {
                app.on_key(key);
            }
        }
    }

    Ok(())
}
