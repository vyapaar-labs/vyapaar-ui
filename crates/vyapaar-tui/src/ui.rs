//! Ratatui UI drawing routines for the Vyapaar workstation.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Row, Table, Tabs, Wrap};

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
}

fn draw_header(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let mode_str = if app.is_input_mode {
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
            Style::default().fg(Color::Black).bg(Color::DarkGray),
        )
    };

    let title_line = Line::from(vec![
        Span::styled(
            " व्यापार ・ Vyapaar Terminal Workstation ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | Model: "),
        Span::styled(&app.model_name, Style::default().fg(Color::Green)),
        Span::raw(" | "),
        mode_str,
    ]);

    let paragraph = Paragraph::new(title_line)
        .style(Style::default().bg(Color::Reset))
        .alignment(Alignment::Left);
    frame.render_widget(paragraph, area);
}

fn draw_tabs(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let titles: Vec<Line> = ActiveTab::ALL
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let num = i + 1;
            Line::from(format!(" {num}: {} ", t.title()))
        })
        .collect();

    let current_idx = ActiveTab::ALL
        .iter()
        .position(|&t| t == app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .select(current_idx)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
        )
        .divider(Span::raw("│"));

    frame.render_widget(tabs, area);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &TuiApp) {
    match app.active_tab {
        ActiveTab::AgentChat => draw_agent_chat(frame, area, app),
        ActiveTab::Watchlist => draw_watchlist(frame, area, app),
        ActiveTab::Screener => draw_placeholder(
            frame,
            area,
            "Neural AST Screener",
            "Screening NSE/BSE universe for technical & fundamental criteria...",
        ),
        ActiveTab::Portfolio => draw_placeholder(
            frame,
            area,
            "Portfolio Snapshot",
            "Tracking holdings, P&L, realized/unrealized returns, and margin utilization...",
        ),
        ActiveTab::Orders => draw_placeholder(
            frame,
            area,
            "Live Orders",
            "Real-time order manager (OMS) for Zerodha, Upstox, Fyers, and AngelOne...",
        ),
        ActiveTab::Backtest => draw_placeholder(
            frame,
            area,
            "Backtest Analytics",
            "Deterministic backtest execution, walk-forward analysis, and Sharpe/DSR metrics...",
        ),
        ActiveTab::Algos => draw_placeholder(
            frame,
            area,
            "Algorithm Management",
            "SEBI-compliant algo identifiers, strategy lifecycle, and parameter tuning...",
        ),
    }
}

fn draw_agent_chat(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    let transcript_area = chunks[0];
    let input_area = chunks[1];

    // Transcript
    let mut lines = Vec::new();
    if app.chat.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(
                "No conversation yet. ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(
                "Press 'i' or Enter to type a prompt to the AI agent.",
                Style::default().fg(Color::Cyan),
            ),
        ]));
    } else {
        for msg in &app.chat.messages {
            lines.extend(message_to_lines(msg));
        }
    }

    let loading_indicator = if app.is_loading {
        " ⚡ Generating... "
    } else {
        ""
    };
    let transcript_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(format!(
            " Agent Chat — SmolLM2 Embedded SLM{loading_indicator}"
        ));

    let total_lines = lines.len();
    let visible_height = transcript_area.height.saturating_sub(2) as usize;
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
        " Prompt (Enter: Send, Esc: Normal mode) "
    } else {
        " Prompt (Press 'i' or Enter to type) "
    };

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(input_border_color))
        .title(input_title);

    let input_text = if app.is_input_mode {
        format!("> {}▋", app.input_buffer)
    } else if app.input_buffer.is_empty() {
        "> [Type 'i' to ask questions to the agent...]".to_string()
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
            "24,852.15",
            "+142.30 (+0.58%)",
            "324.5M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "NIFTY BANK",
            "Index",
            "52,310.80",
            "+385.10 (+0.74%)",
            "182.1M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "RELIANCE",
            "NSE Equity",
            "2,745.50",
            "-12.20 (-0.44%)",
            "6.4M",
        ])
        .style(Style::default().fg(Color::Red)),
        Row::new(vec![
            "HDFCBANK",
            "NSE Equity",
            "1,780.25",
            "+18.60 (+1.06%)",
            "14.2M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "INFY",
            "NSE Equity",
            "1,895.00",
            "+31.50 (+1.69%)",
            "8.9M",
        ])
        .style(Style::default().fg(Color::Green)),
        Row::new(vec![
            "TCS",
            "NSE Equity",
            "4,120.30",
            "+15.00 (+0.37%)",
            "2.1M",
        ])
        .style(Style::default().fg(Color::Green)),
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
            "  Tip: Switch to the Agent tab (press '7' or Tab) to chat with the embedded LLM.",
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
