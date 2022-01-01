#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Clone)]
pub struct Form {
    id: String,
    pub title: Option<String>,
    fields: Vec<SharedFocusable>,
    selected_index: Option<usize>,
}
/// Form with editable content
impl Form {
    pub fn new(id: &str, title: Option<String>, fields: Vec<SharedFocusable>) -> Form {
        Form {
            id: String::from(id),
            title,
            fields,
            selected_index: None,
        }
    }
    pub fn add_field(&mut self, field: SharedFocusable) {
        self.fields.push(field);
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(String::from(title));
    }
    pub fn remove_title(&mut self) {
        self.title = None;
    }
}

impl Component for Form {
    fn name(&self) -> &str {
        "Form"
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunk = get_modal_rect(chunk);
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(block_style())
            .border_type(BorderType::Rounded);
        let modal = match &self.title {
            Some(title) => modal.title(title.clone()),
            None => modal,
        };
        let text = vec![Spans::from(Span::styled(
            String::from("Form Placeholder"),
            paragraph_style(),
        ))];
        let paragraph = Paragraph::new(text)
            .block(modal)
            .style(paragraph_style())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });

        parent.render_widget(paragraph, chunk);

        Ok(())
    }

    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        _router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Esc => {
                return Ok(Propagate);
            }
            KeyCode::Enter => {
                return Ok(Propagate);
            }
            KeyCode::Char(c) => {
                return Ok(Propagate);
            }
            _ => Ok(Propagate),
        }
    }
}
pub fn block_style() -> Style {
    Style::default().bg(Color::DarkGray).fg(Color::White)
}

pub fn paragraph_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}
