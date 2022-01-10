pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub use std::{cell::RefCell, sync::Arc};

pub use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

pub fn get_modal_rect(parent: Rect) -> Rect {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(parent);
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(vertical_chunks[1]);

    let center = horizontal_chunks[1];
    center
}

pub fn get_padded_rect(parent: Rect, margin: u16) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(margin)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(parent);

    let center = chunks[0];
    center
}
