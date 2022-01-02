#![allow(unused_variables)]
#![allow(dead_code)]

use crate::app::ui::*;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
pub struct Modal {
    pub title: String,
    pub text: String,
    active: bool,
}
/// Modal with editable content
impl Modal {
    pub fn new(title: &str, text: &str) -> Modal {
        Modal {
            title: String::from(title),
            text: String::from(text),
            active: true,
        }
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = String::from(title);
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    pub fn write(&mut self, c: char) {
        self.text.push(c);
    }
    pub fn backspace(&mut self) {
        self.text.pop();
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Component for Modal {
    fn name(&self) -> &str {
        "Modal"
    }
    fn id(&self) -> String {
        self.text.clone()
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
            .title(self.title.clone())
            .border_type(BorderType::Rounded);

        let text = Text::from(self.text.clone());
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
                self.deactivate();
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
