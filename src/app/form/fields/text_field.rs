#![allow(unused_imports)]
#![allow(unused_variables)]
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

#[derive(Debug, Clone)]
pub struct TextField {
    pub title: Option<String>,
    pub value: String,
    pub id: String,
    buf: String, // editable buffer
    focused: bool,
    read_only: bool,
}
/// TextField with editable content
impl TextField {
    pub fn new(id: &str, title: &str, value: &str, read_only: bool) -> TextField {
        TextField {
            id: String::from(id),
            title: Some(String::from(title)),
            value: String::from(value),
            focused: false,
            buf: String::new(),
            read_only: read_only,
        }
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(String::from(title));
    }
    pub fn remove_title(&mut self) {
        self.title = None;
    }
    pub fn set_value(&mut self, value: &str) {
        self.value = String::from(value);
    }
    pub fn write(&mut self, c: char) {
        self.buf.push(c);
    }
    pub fn backspace(&mut self) {
        self.buf.pop();
    }
}
impl Component for TextField {
    fn name(&self) -> &str {
        "TextField"
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
            self.value.clone(),
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
            KeyCode::Backspace => {
                self.backspace();
                Ok(Propagate)
            }
            KeyCode::Esc => {
                self.blur();
                return Ok(Propagate);
            }
            KeyCode::Enter => {
                self.write('\n');
                return Ok(Propagate);
            }
            KeyCode::Char(c) => {
                self.write(c);
                Ok(Refresh)
            }
            _ => Ok(Propagate),
        }
    }
}
impl Focusable for TextField {
    fn tab_index(&self) -> usize {
        0
    }
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn blur(&mut self) {
        self.focused = false;
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
