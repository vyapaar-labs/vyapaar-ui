//! Layout model: how the terminal splits into header, tabs, content, footer.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// The vertical layout regions of the terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutModel {
    /// Header strip (brand + system status).
    pub header: Rect,
    /// Tab bar.
    pub tabs: Rect,
    /// Main content area.
    pub content: Rect,
    /// Footer / status bar.
    pub footer: Rect,
}

impl LayoutModel {
    /// Compute the layout for the whole terminal area.
    pub fn new(area: Rect) -> Self {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);
        Self {
            header: rows[0],
            tabs: rows[1],
            content: rows[2],
            footer: rows[3],
        }
    }

    /// The content area split into a left panel and right panel.
    pub fn content_columns(&self) -> [Rect; 2] {
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(self.content);
        [cols[0], cols[1]]
    }
}
