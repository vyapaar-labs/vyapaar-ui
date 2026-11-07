//! Ratatui UI drawing routines for the Vyapaar workstation.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Table, Tabs, Wrap};

use vyapaar_highlight::HighlightClass;

use crate::agent_chat::{AgentMessage, AgentRole};
use crate::app::{ActiveTab, TuiApp};
use crate::layout::LayoutModel;

/// Map a syntax highlight lexical class to a Ratatui style.
pub fn class_to_style(class: HighlightClass) -> Style {
    match class {
        HighlightClass::Keyword => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        HighlightClass::String => Style::default().fg(Color::Green),
        HighlightClass::Comment => Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::ITALIC),
        HighlightClass::Number => Style::default().fg(Color::Yellow),
        HighlightClass::Function => Style::default()
            .fg(Color::LightBlue)
            .add_modifier(Modifier::BOLD),
        HighlightClass::Type => Style::default().fg(Color::LightCyan),
        HighlightClass::Punctuation => Style::default().fg(Color::White),
        HighlightClass::Plain => Style::default().fg(Color::Reset),
    }
}

/// Convert an AgentMessage with highlights into Ratatui Lines.
pub fn message_to_lines(msg: &AgentMessage) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let (badge, badge_style) = match msg.role {
        AgentRole::User => (
            "[User] ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        AgentRole::Agent => (
            "[Vyapaar] ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
        AgentRole::System => (
            "[System] ",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::DIM),
        ),
    };

    lines.push(Line::from(vec![Span::styled(badge, badge_style)]));

    if msg.text.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "⚡ Thinking...",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )]));
    } else {
        for l in msg.text.lines() {
            lines.push(Line::from(Span::raw(l.to_string())));
        }
    }

    lines.push(Line::from(""));
    lines
}

/// Draw the entire workstation UI onto a Ratatui Frame.
pub fn draw(frame: &mut Frame, app: &TuiApp) {
    let layout = LayoutModel::new(frame.area());

    // 1. Header
    draw_header(frame, layout.header, app);

    // 2. Tabs
    draw_tabs(frame, layout.tabs, app);

    // 3. Content
    draw_content(frame, layout.content, app);

    // 4. Footer
    draw_footer(frame, layout.footer, app);

    // 5. Palette overlay
    if app.is_palette_open {
        draw_palette(frame, frame.area(), app);
    }
}

fn draw_header(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let mode_str = if app.is_palette_open {
        Span::styled(
            " [COMMAND PALETTE] ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        )
    } else if app.is_input_mode {
        Span::styled(
            " [CHAT INPUT] ",
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            " [NORMAL] ",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::DIM),
        )
    };

    let title_line = Line::from(vec![
        Span::styled(
            "व्यापार・Vyapaar",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Indian Equities Quantitative Workstation "),
        mode_str,
        Span::styled(
            format!(" | LLM: {}", app.model_name),
            Style::default().fg(Color::Green),
        ),
    ]);

    let p = Paragraph::new(title_line).alignment(Alignment::Left);
    frame.render_widget(p, area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let titles: Vec<Line> = ActiveTab::ALL
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let num = i + 1;
            Line::from(format!(" {num}:{tab_title} ", tab_title = tab.title()))
        })
        .collect();

    let selected_tab_index = ActiveTab::ALL
        .iter()
        .position(|&t| t == app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(selected_tab_index)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::raw("|"));

    frame.render_widget(tabs, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &TuiApp) {
    match app.active_tab {
        ActiveTab::Watchlist => draw_watchlist(frame, area, app),
        ActiveTab::Screener => draw_placeholder(
            frame,
            area,
            "Neural AST Screener",
            "Vectorized query filter over 4,000 Arrow fields (P/E, ROCE, RSI, 52W).",
        ),
        ActiveTab::Portfolio => draw_placeholder(
            frame,
            area,
            "Portfolio Snapshot",
            "FIFO tax-lots, unrealized STCG/LTCG, and real-time MTM valuation.",
        ),
        ActiveTab::Orders => draw_placeholder(
            frame,
            area,
            "Orders & OMS",
            "Live order book state machine with SEBI pre-trade risk validation.",
        ),
        ActiveTab::Backtest => draw_placeholder(
            frame,
            area,
            "Backtest Results",
            "Walk-forward efficiency, Sharpe ratio, Deflated Sharpe, and PBO metrics.",
        ),
        ActiveTab::Algos => draw_placeholder(
            frame,
            area,
            "Algos & Strategy Management",
            "Deploy, verify lookahead bias, pause, and monitor live trading algorithms.",
        ),
        ActiveTab::AgentChat => draw_agent_chat(frame, area, app),
    }
}

fn draw_agent_chat(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let transcript_area = chunks[0];
    let input_area = chunks[1];

    // Transcript Box
    let transcript_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Quantcept Research Desk (Live LLM Stream) ");

    let mut lines = Vec::new();
    for msg in &app.chat.messages {
        lines.extend(message_to_lines(msg));
    }

    let visible_height = transcript_area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();
    let scroll_y = if total_lines > visible_height {
        (total_lines - visible_height) as u16
    } else {
        0
    };

    let transcript_p = Paragraph::new(lines)
        .block(transcript_block)
        .scroll((scroll_y, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(transcript_p, transcript_area);

    // Input Box
    let input_border_color = if app.is_input_mode {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let input_title = if app.is_input_mode {
        " Prompt (Enter: Send, Esc: Normal mode, '/' or Ctrl+P: Palette) "
    } else {
        " Prompt (Press 'i' or Enter to type, Ctrl+P: Palette) "
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(input_border_color))
        .title(input_title);

    let input_text = if app.is_input_mode {
        format!("> {}▋", app.input_buffer)
    } else if app.input_buffer.is_empty() {
        "> [Press 'i' to type prompt, '/' or Ctrl+P for slash commands...]".to_string()
    } else {
        format!("> {}", app.input_buffer)
    };

    let input_style = if app.is_input_mode {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input_p = Paragraph::new(input_text)
        .block(input_block)
        .style(input_style);
    frame.render_widget(input_p, input_area);
}

fn draw_palette(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let popup_width = 64.min(area.width.saturating_sub(4));
    let popup_height = 16.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Command Palette (Up/Down: Navigate, Enter: Run, Esc: Close) ");

    let filtered = app.filtered_palette();
    let mut items = Vec::new();

    items.push(Line::from(vec![
        Span::styled("> ", Style::default().fg(Color::Cyan)),
        Span::styled(&app.palette_query, Style::default().fg(Color::White)),
        Span::styled("▋", Style::default().fg(Color::DarkGray)),
    ]));
    items.push(Line::from(
        "─".repeat(popup_width.saturating_sub(2) as usize),
    ));

    if filtered.is_empty() {
        items.push(Line::from("  No matching slash commands."));
    } else {
        for (idx, (cmd, desc)) in filtered.iter().enumerate() {
            let is_sel = idx == app.palette_selected;
            let style = if is_sel {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_sel { "▶ " } else { "  " };
            items.push(Line::from(vec![
                Span::styled(format!("{prefix}{:<13} ", cmd), style),
                Span::styled(
                    *desc,
                    if is_sel {
                        style
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
            ]));
        }
    }

    let p = Paragraph::new(items).block(block);
    frame.render_widget(p, popup_area);
}

fn draw_watchlist(frame: &mut Frame, area: Rect, _app: &TuiApp) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Watchlist — Indian Equities & Indices (NSE) ");

    let header = Row::new(vec!["Symbol", "Instrument", "LTP (₹)", "Change", "Volume"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let rows = vec![
        Row::new(vec![
            "NIFTY 50",
            "Index",
            "24,148.20",
            "+124.50 (+0.52%)",
            "482.1M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "BANKNIFTY",
            "Index",
            "51,885.10",
            "-78.20 (-0.15%)",
            "215.4M",
        ])
        .style(Style::default().fg(Color::Red)),
        Row::new(vec![
            "RELIANCE",
            "NSE Equity",
            "2,980.50",
            "+34.00 (+1.15%)",
            "8.4M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "TCS",
            "NSE Equity",
            "4,120.00",
            "-12.50 (-0.30%)",
            "2.9M",
        ])
        .style(Style::default().fg(Color::Red)),
        Row::new(vec![
            "HDFCBANK",
            "NSE Equity",
            "1,648.75",
            "+5.10 (+0.31%)",
            "14.2M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "INFY",
            "NSE Equity",
            "1,822.30",
            "-4.40 (-0.24%)",
            "6.1M",
        ])
        .style(Style::default().fg(Color::Red)),
        Row::new(vec![
            "ICICIBANK",
            "NSE Equity",
            "1,265.40",
            "+8.30 (+0.66%)",
            "11.7M",
        ])
        .style(Style::default().fg(Color::Green)),
    ];

    let widths = [
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(14),
        Constraint::Length(22),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths).header(header).block(block);
    frame.render_widget(table, area);
}

fn draw_placeholder(frame: &mut Frame, area: Rect, title: &str, description: &str) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(" {title} "));

    let text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {title}"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {description}"),
            Style::default().fg(Color::White),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Tip: Switch to the Agent tab (press '7' or Tab) to chat, or Ctrl+P for slash commands.",
            Style::default().fg(Color::DarkGray),
        )]),
    ];

    let p = Paragraph::new(text).block(block);
    frame.render_widget(p, area);
}

fn draw_footer(frame: &mut Frame, area: Rect, _app: &TuiApp) {
    let footer_text = Line::from(vec![
        Span::styled(" [Tab / 1-7] ", Style::default().fg(Color::Yellow)),
        Span::raw("Tabs | "),
        Span::styled("[Ctrl+P] ", Style::default().fg(Color::Magenta)),
        Span::raw("Palette | "),
        Span::styled("[i / Enter] ", Style::default().fg(Color::Yellow)),
        Span::raw("Chat Input | "),
        Span::styled("[Esc] ", Style::default().fg(Color::Yellow)),
        Span::raw("Normal Mode | "),
        Span::styled("[q] ", Style::default().fg(Color::Yellow)),
        Span::raw("Quit"),
    ]);

    let p = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center);
    frame.render_widget(p, area);
}
