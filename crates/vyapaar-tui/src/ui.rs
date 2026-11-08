//! Ratatui UI drawing routines for the Vyapaar workstation.

use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Row, Sparkline, Table, Tabs, Wrap};
use rust_decimal::Decimal;
use vyapaar_ui_core::algos::AlgoStatus;

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
            format!(" | Persona: {}", app.model_name),
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
        ActiveTab::Backtest => draw_backtest(frame, area, app),
        ActiveTab::Algos => draw_algos(frame, area, app),
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
        " Prompt (Press 'i' or Enter to type, '/' or Ctrl+P: Palette) "
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
    let popup_width = 72.min(area.width.saturating_sub(4));
    let popup_height = 18.min(area.height.saturating_sub(4));

    let x = (area.width.saturating_sub(popup_width)) / 2;
    let y = (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Command Palette ([Enter] Run  [Tab] Complete  [Esc] Dismiss  [Up/Down] Move) ");

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
        items.push(Line::from("  No matching slash commands found."));
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
                Span::styled(format!("{prefix}{:<15} ", cmd), style),
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
            "  Tip: Switch to the Agent tab (press '7' or Tab) to chat, or press '/' / Ctrl+P for slash commands.",
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
        Span::styled("[/ or Ctrl+P] ", Style::default().fg(Color::Magenta)),
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


fn draw_algos(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let vert_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(area);

    let main_area = vert_chunks[0];
    let action_bar_area = vert_chunks[1];

    let horiz_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(main_area);

    let left_area = horiz_chunks[0];
    let right_area = horiz_chunks[1];

    // Left Panel: Table of registered algorithms
    let header = Row::new(vec!["Algorithm", "Version", "Status", "Sharpe", "Instruments"]).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let mut rows = Vec::new();
    for (idx, algo) in app.algos.iter().enumerate() {
        let is_selected = idx == app.selected_algo_index;
        let (status_str, _status_color) = match algo.status {
            AlgoStatus::Active => ("● Active", Color::Green),
            AlgoStatus::Paused => ("❚❚ Paused", Color::Yellow),
            AlgoStatus::Stopped => ("■ Stopped", Color::Red),
            AlgoStatus::Error => ("✖ Error", Color::LightRed),
        };

        let inst_str = algo.descriptor.target_instruments.join(", ");
        let name_display = if is_selected {
            format!("▶ {}", algo.descriptor.name)
        } else {
            format!("  {}", algo.descriptor.name)
        };

        let row_style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        rows.push(
            Row::new(vec![
                name_display,
                algo.descriptor.version.clone(),
                status_str.to_string(),
                format!("{:.2}", algo.baseline_sharpe),
                inst_str,
            ])
            .style(row_style),
        );
    }

    let left_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Registered Algorithms (60%) ");

    let widths = [
        Constraint::Percentage(32),
        Constraint::Percentage(12),
        Constraint::Percentage(16),
        Constraint::Percentage(12),
        Constraint::Percentage(28),
    ];

    let table = Table::new(rows, widths).header(header).block(left_block);
    frame.render_widget(table, left_area);

    // Right Panel: Selected Strategy Details & Parameter Inspector
    let right_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Selected Strategy & Parameter Inspector (40%) ");

    let mut details_lines = Vec::new();
    if let Some(selected) = app.selected_algo() {
        details_lines.push(Line::from(vec![
            Span::styled("Strategy: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&selected.descriptor.name, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({})", selected.descriptor.id), Style::default().fg(Color::DarkGray)),
        ]));
        details_lines.push(Line::from(vec![
            Span::styled("Version:  ", Style::default().fg(Color::Cyan)),
            Span::styled(&selected.descriptor.version, Style::default().fg(Color::White)),
            Span::styled(" | Origin: ", Style::default().fg(Color::Cyan)),
            Span::styled(selected.descriptor.source.as_str(), Style::default().fg(Color::Yellow)),
        ]));
        details_lines.push(Line::from(vec![
            Span::styled("Status:   ", Style::default().fg(Color::Cyan)),
            Span::styled(selected.status.as_str(), Style::default().fg(match selected.status {
                AlgoStatus::Active => Color::Green,
                AlgoStatus::Paused => Color::Yellow,
                _ => Color::Red,
            })),
            Span::styled(format!(" | Baseline Sharpe: {:.2}", selected.baseline_sharpe), Style::default().fg(Color::White)),
        ]));
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(vec![
            Span::styled("Description: ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            Span::styled(&selected.descriptor.description, Style::default().fg(Color::White)),
        ]));
        details_lines.push(Line::from(""));
        details_lines.push(Line::from(Span::styled("─── Parameter Search Contract ──────────────────────────", Style::default().fg(Color::DarkGray))));

        for p in &selected.descriptor.parameters {
            let range_str = if let (Some(min), Some(max), Some(step)) = (p.min, p.max, p.step) {
                format!("{min} .. {max} (step {step})")
            } else {
                "Fixed (no sweep)".to_string()
            };
            details_lines.push(Line::from(vec![
                Span::styled(format!(" • {:<15} ", p.name), Style::default().fg(Color::LightBlue).add_modifier(Modifier::BOLD)),
                Span::styled(format!("Default: {:<5} ", p.default), Style::default().fg(Color::White)),
                Span::styled(format!("[{range_str}]"), Style::default().fg(Color::Yellow)),
            ]));
            if !p.description.is_empty() {
                details_lines.push(Line::from(Span::styled(format!("   ↳ {}", p.description), Style::default().fg(Color::DarkGray))));
            }
        }

        details_lines.push(Line::from(""));
        details_lines.push(Line::from(vec![
            Span::styled("Target Instruments: ", Style::default().fg(Color::DarkGray)),
            Span::styled(selected.descriptor.target_instruments.join(", "), Style::default().fg(Color::Green)),
        ]));
    } else {
        details_lines.push(Line::from("  No strategy selected."));
    }

    let p = Paragraph::new(details_lines)
        .block(right_block)
        .wrap(Wrap { trim: true });
    frame.render_widget(p, right_area);

    // Action Bar
    let action_line = Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Backtest | "),
        Span::styled("[T] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Tune Params | "),
        Span::styled("[S] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Suggest Trade | "),
        Span::styled("[Space] ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Toggle Status | "),
        Span::styled("[↑/↓ or j/k] ", Style::default().fg(Color::Cyan)),
        Span::raw("Navigate"),
    ]);
    let action_bar = Paragraph::new(action_line).alignment(Alignment::Center);
    frame.render_widget(action_bar, action_bar_area);
}

fn draw_backtest(frame: &mut Frame, area: Rect, app: &TuiApp) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let top_row = rows[0];
    let bottom_row = rows[1];

    let top_quads = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(top_row);

    let bottom_quads = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(bottom_row);

    let q1_area = top_quads[0];
    let q2_area = top_quads[1];
    let q3_area = bottom_quads[0];
    let q4_area = bottom_quads[1];

    // Q1: Top Left - KPI Cards
    let q1_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Key Performance Indicators (KPIs) ");

    let mut kpi_lines = Vec::new();
    if let Some(ref r) = app.backtest_report {
        let pnl_color = if r.net_pnl >= Decimal::ZERO { Color::Green } else { Color::Red };
        let pnl_sign = if r.net_pnl >= Decimal::ZERO { "+" } else { "" };
        let ret_sign = if r.total_return_pct >= Decimal::ZERO { "+" } else { "" };

        kpi_lines.push(Line::from(""));
        kpi_lines.push(Line::from(vec![
            Span::styled("  Initial Capital: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", r.initial_capital), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled("   Final Equity: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", r.final_equity), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        ]));
        kpi_lines.push(Line::from(""));
        kpi_lines.push(Line::from(vec![
            Span::styled("  Net Profit/Loss: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}{:.2}", pnl_sign, r.net_pnl), Style::default().fg(pnl_color).add_modifier(Modifier::BOLD)),
            Span::styled("   Total Return: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}{:.2}%", ret_sign, r.total_return_pct), Style::default().fg(pnl_color).add_modifier(Modifier::BOLD)),
        ]));
        kpi_lines.push(Line::from(""));
        kpi_lines.push(Line::from(vec![
            Span::styled("  Sharpe Ratio:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2}", r.sharpe_ratio), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled("   Max Drawdown: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2}%", r.max_drawdown_pct), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]));
        kpi_lines.push(Line::from(""));
        kpi_lines.push(Line::from(vec![
            Span::styled("  Total Trades:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", r.total_trades), Style::default().fg(Color::White)),
            Span::styled("         Win Rate:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2}%", r.win_rate_pct), Style::default().fg(Color::Green)),
            Span::styled(" | Profit Factor: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{:.2}", r.profit_factor), Style::default().fg(Color::White)),
        ]));
    } else {
        kpi_lines.push(Line::from("  No backtest loaded. Press [Enter] on an algorithm or type /backtest."));
    }
    let p1 = Paragraph::new(kpi_lines).block(q1_block);
    frame.render_widget(p1, q1_area);

    // Q2: Top Right - Parameter Delta Comparison Table
    let q2_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Parameter Delta (Default vs Tuned) ");

    let header2 = Row::new(vec!["Parameter", "Default", "Tuned Value", "Status"]).style(
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    );
    let mut rows2 = Vec::new();
    if let Some(ref opt) = app.optimization_result {
        for (k, v_tuned) in &opt.tuned_params {
            let v_def = opt.default_params.get(k).copied().unwrap_or(*v_tuned);
            let is_diff = v_tuned != &v_def;
            let val_style = if is_diff {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            rows2.push(Row::new(vec![
                k.clone(),
                format!("{v_def}"),
                format!("{v_tuned}"),
                if is_diff { "Tuned (Optimal)".to_string() } else { "Unchanged".to_string() },
            ]).style(val_style));
        }
    }
    let widths2 = [
        Constraint::Percentage(35),
        Constraint::Percentage(20),
        Constraint::Percentage(25),
        Constraint::Percentage(20),
    ];
    let t2 = Table::new(rows2, widths2).header(header2).block(q2_block);
    frame.render_widget(t2, q2_area);

    // Q3: Bottom Left - ASCII / Braille Equity Curve & Drawdown Chart (Sparkline)
    let q3_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(" Analytical Equity Curve (Sparkline) ");

    let sparkline_data = &app.backtest_equity_curve;
    let sparkline = Sparkline::default()
        .block(q3_block)
        .style(Style::default().fg(Color::Green))
        .data(sparkline_data);
    frame.render_widget(sparkline, q3_area);

    // Q4: Bottom Right - Optimal Trade Suggestion Card
    let q4_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" Optimal Trade Suggestion ");

    let mut sugg_lines = Vec::new();
    if let Some(ref s) = app.trade_suggestion {
        sugg_lines.push(Line::from(vec![
            Span::styled("Asset: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&s.symbol, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" | Bias: ", Style::default().fg(Color::DarkGray)),
            Span::styled(s.signal.bias.as_str(), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        sugg_lines.push(Line::from(vec![
            Span::styled("Trigger: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&s.signal.reason, Style::default().fg(Color::Yellow)),
        ]));
        sugg_lines.push(Line::from(vec![
            Span::styled("Entry: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", s.signal.entry_price), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" | Stop Loss: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", s.signal.stop_loss), Style::default().fg(Color::Red)),
        ]));
        sugg_lines.push(Line::from(vec![
            Span::styled("Target 1 (1:2): ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", s.signal.target_1), Style::default().fg(Color::Green)),
            Span::styled(" | Target 2 (1:3): ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", s.signal.target_2), Style::default().fg(Color::Green)),
        ]));
        sugg_lines.push(Line::from(vec![
            Span::styled("Sized Qty: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{} units", s.sizing.quantity), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" | Margin: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2} ({:.1}%)", s.sizing.notional_value, s.sizing.margin_utilization_pct), Style::default().fg(Color::White)),
        ]));
        sugg_lines.push(Line::from(vec![
            Span::styled("Friction Fee: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("₹{:.2}", s.statutory_charges.total_charges), Style::default().fg(Color::DarkGray)),
            Span::styled(" | Net R:R: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("1 : {:.2}", s.statutory_charges.net_risk_reward_ratio), Style::default().fg(Color::LightGreen).add_modifier(Modifier::BOLD)),
        ]));
    } else {
        sugg_lines.push(Line::from("  No trade suggestion computed. Press [S] on an algo or type /suggest."));
    }
    let p4 = Paragraph::new(sugg_lines).block(q4_block).wrap(Wrap { trim: true });
    frame.render_widget(p4, q4_area);
}
