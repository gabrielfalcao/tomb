#![allow(unused_variables)]
#![allow(dead_code)]
use super::super::super::ui::*;
use super::text_field::TextField;
use crate::aes256cbc::Key;
use crate::app::log_error;
use crate::tomb::{AES256Secret, AES256Tomb};

use crate::ironpunk::*;

use crossterm::event::KeyEvent;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Rect},
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Clone)]
pub struct SecretField {
    pub field: TextField,
    pub secret: Option<AES256Secret>,
    pub tomb: AES256Tomb,
    pub key: Key,
    pub visible: bool,
}

impl SecretField {
    pub fn new(
        id: &str,
        title: &str,
        read_only: bool,
        visible: bool,
        secret: Option<AES256Secret>,
        tomb: AES256Tomb,
        key: Key,
    ) -> SecretField {
        SecretField {
            field: TextField::new(id, title, String::new(), read_only, visible),
            secret,
            tomb,
            key,
            visible,
        }
    }
    pub fn set_secret(&mut self, secret: Option<AES256Secret>) {
        self.secret = secret;
    }
    pub fn set_title(&mut self, title: &str) {
        self.field.set_title(title)
    }
    pub fn remove_title(&mut self) {
        self.field.remove_title()
    }
    pub fn write(&mut self, c: char) {
        self.field.write(c)
    }
    pub fn backspace(&mut self) {
        self.field.backspace()
    }

    pub fn get_plaintext(&mut self) -> Result<String, Error> {
        let secret = match &self.secret {
            Some(secret) => secret.clone(),
            None => return Err(Error::with_message(format!("no secret selected"))),
        };

        match self.tomb.get_string(secret.path.as_str(), self.key.clone()) {
            Ok(secret) => Ok(secret),
            Err(err) => return Err(Error::with_message(format!("{}", err))),
        }
    }
}

impl Component for SecretField {
    fn name(&self) -> &str {
        "SecretField"
    }
    fn id(&self) -> String {
        self.field.id()
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let style = if self.field.focused {
            block_style().fg(if self.visible {
                color_error_fg()
            } else {
                color_light()
            })
        } else {
            if self.visible {
                default_style().fg(color_light())
            } else {
                block_style()
            }
        };
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(style)
            .border_type(BorderType::Thick);
        let modal = match &self.field.title {
            Some(title) => modal.title(title.clone()),
            None => modal,
        };
        let text = Text::from(self.get_value());
        let paragraph = Paragraph::new(text)
            .block(modal)
            .style(style)
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
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.field
            .process_keyboard(event, terminal, context, router)
    }
}
impl Focusable for SecretField {
    fn tab_index(&self) -> usize {
        self.field.tab_index()
    }
    fn is_focused(&self) -> bool {
        self.field.is_focused()
    }
    fn focus(&mut self) {
        self.field.focus()
    }
    fn blur(&mut self) {
        self.field.blur()
    }
}

impl Field for SecretField {
    fn write(&mut self, c: char) {
        self.field.write(c)
    }
    fn backspace(&mut self) {
        self.field.backspace()
    }

    fn get_id(&self) -> String {
        self.field.id()
    }
    fn set_value(&mut self, value: &str) {
        let secret = match &self.secret {
            Some(secret) => secret.clone(),
            None => return,
        };

        let path = secret.path.clone();
        let plaintext = String::from(value);

        match self
            .tomb
            .add_secret(path.as_str(), plaintext.clone(), self.key.clone())
        {
            Ok(_) => {
                if self.visible {
                    self.field.set_value(plaintext.as_str());
                } else {
                    self.field.set_value(secret.value.as_str());
                }
            }
            Err(error) => {
                log_error(format!("error setting secret to field {}: {}", path, error));
            }
        }
    }
    fn get_value(&mut self) -> String {
        let secret = match &self.secret {
            Some(secret) => secret.clone(),
            None => return String::from("<not set>"),
        };

        if !self.visible {
            return secret.value.clone();
        }
        match self.get_plaintext() {
            Ok(plaintext) => plaintext.clone(),
            Err(error) => {
                log_error(format!(
                    "error setting plaintext value into field {}",
                    error
                ));
                secret.value.clone()
            }
        }
    }
    fn constraint(&self) -> Constraint {
        self.field.constraint()
    }
    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.field.set_visible(visible)
    }
    fn get_visible(&mut self) -> bool {
        self.visible
    }
    fn get_title(&mut self) -> Option<String> {
        self.field.get_title()
    }
    fn set_title(&mut self, title: &str) {
        self.field.set_title(title)
    }
}
