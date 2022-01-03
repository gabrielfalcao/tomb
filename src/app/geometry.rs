pub use super::state::*;

pub use super::components::{menu::Menu, modal::Modal};

use crate::ironpunk::*;

extern crate clipboard;

use tui::layout::{Constraint, Direction, Layout};

pub fn vertical_stack(size: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(size);

    let header = chunks[0];
    let body = chunks[1];
    let footer = chunks[2];

    (header, body, footer)
}

pub fn body_sides(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(size);
    let left = chunks[0];
    let right = chunks[1];
    (left, right)
}

pub fn horizontal_split(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);
    let left = chunks[0];
    let right = chunks[1];
    (left, right)
}
