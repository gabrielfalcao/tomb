#![allow(unused_variables)]
#![allow(dead_code)]
use crate::app::log_error;
use crate::app::ui::*;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Rect},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
pub struct TextField {
    pub title: Option<String>,
    pub value: String,
    pub id: String,
    pub focused: bool,
    pub visible: bool,
    pub read_only: bool,
}

impl TextField {
    pub fn new(id: &str, title: &str, value: String, read_only: bool, visible: bool) -> TextField {
        TextField {
            id: String::from(id),
            title: Some(String::from(title)),
            value: value.clone(),
            focused: false,
            read_only,
            visible,
        }
    }
    pub fn remove_title(&mut self) {
        self.title = None;
    }
    pub fn write(&mut self, c: char) {
        self.value.push(c);
    }
    pub fn backspace(&mut self) {
        self.value.pop();
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
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(if self.focused {
                block_style().fg(color_light())
            } else {
                block_style()
            })
            .border_type(BorderType::Thick);
        let modal = match &self.title {
            Some(title) => modal.title(title.clone()),
            None => modal,
        };
        let text = Text::from(self.get_value());
        let paragraph = Paragraph::new(text)
            .block(modal)
            .style(if self.focused {
                paragraph_style().fg(color_light())
            } else {
                paragraph_style()
            })
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
        log_error(format!("selected field: {:?}", self.get_id()));
    }
    fn blur(&mut self) {
        self.focused = false;
    }
}

impl Field for TextField {
    fn get_id(&self) -> String {
        self.id.clone()
    }
    fn get_title(&mut self) -> Option<String> {
        self.title.clone()
    }
    fn set_title(&mut self, title: &str) {
        self.title = Some(String::from(title));
    }

    fn set_value(&mut self, value: &str) {
        self.value = String::from(value);
    }
    fn get_value(&mut self) -> String {
        self.value.clone()
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    fn get_visible(&mut self) -> bool {
        self.visible
    }
    fn constraint(&self) -> Constraint {
        Constraint::Length(3)
    }
}
