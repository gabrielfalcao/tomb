#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use super::super::components::{menu::Menu, modal::Modal, searchbox::SearchBox};
pub use super::super::form::{Form, TextField};
use super::super::geometry::*;
use super::super::log_error;
pub use super::super::state::*;
use super::super::ui;
use chrono::prelude::*;

use crate::ironpunk::*;
#[cfg(feature = "osx")]
use mac_notification_sys::*;

extern crate clipboard;
use super::super::{AES256Secret, AES256Tomb, TombConfig};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};

use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Row, Table, Wrap},
    Terminal,
};

pub struct SecretDetails<'a> {
    key: Key,
    tomb: AES256Tomb,
    secret: Option<AES256Secret>,
    tomb_config: TombConfig,
    visible: bool,
    phantom: PhantomData<&'a List<'a>>,
}

impl<'a> SecretDetails<'a> {
    pub fn new(
        key: Key,
        tomb: AES256Tomb,
        secret: Option<AES256Secret>,
        tomb_config: TombConfig,
        visible: bool,
    ) -> SecretDetails<'a> {
        SecretDetails {
            key,
            tomb,
            secret,
            tomb_config,
            visible,
            phantom: PhantomData,
        }
    }
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    pub fn set_secret(&mut self, secret: AES256Secret) {
        self.secret = Some(secret);
    }
    pub fn selected_secret(&mut self) -> Result<AES256Secret, Error> {
        match &self.secret {
            Some(secret) => Ok(secret.clone()),
            None => Err(Error::with_message(format!("no secret selected"))),
        }
    }
    pub fn get_plaintext(&mut self, secret: &AES256Secret) -> Result<String, Error> {
        match self.tomb.get_string(secret.path.as_str(), self.key.clone()) {
            Ok(secret) => Ok(secret),
            Err(err) => return Err(Error::with_message(format!("{}", err))),
        }
    }
    pub fn selected_secret_string(&mut self) -> Result<String, Error> {
        match self.selected_secret() {
            Ok(secret) => self.get_plaintext(&secret),
            Err(err) => Err(err),
        }
    }
    fn create_form(&mut self, secret: AES256Secret) -> Form {
        let digest = secret
            .digest
            .iter()
            .map(|b| format!("{:02x}", *b))
            .collect::<Vec<_>>()
            .join("");

        let mut form = Form::new(
            "secret-details",
            Some(String::from("Details Title")),
            Vec::new(),
        );
        let field_path = TextField::new("secret-path-field", "path", secret.path.clone(), true);
        let field_digest =
            TextField::new("secret-digest-field", "digest", format!("{}", digest), true);
        let field_value = TextField::new(
            "secret-value-field",
            "value",
            match self.visible {
                true => {
                    let secret = secret.clone();
                    match self.get_plaintext(&secret) {
                        Ok(plaintext) => plaintext,
                        Err(err) => format!("{}", err),
                    }
                }
                false => secret.value,
            },
            true,
        );
        let field_notes = TextField::new(
            "secret-notes-field",
            "notes",
            match secret.notes {
                Some(notes) => notes.clone(),
                None => String::from("<none>"),
            },
            true,
        );
        let field_updated_at = TextField::new(
            "secret-updated-at-field",
            "updated-at",
            chrono_humanize::HumanTime::from(secret.updated_at).to_string(),
            true,
        );

        form.add_field(field_digest);
        form.add_field(field_path);
        form.add_field(field_value);
        form.add_field(field_updated_at);
        form.add_field(field_notes);
        form
    }
}

impl Component for SecretDetails<'_> {
    fn name(&self) -> &str {
        "SecretDetails"
    }
    fn id(&self) -> String {
        String::from("SecretDetails")
    }
    fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        Ok(Propagate)
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        match &self.secret {
            Some(secret) => {
                let secret = secret.clone();
                let mut form = self.create_form(secret);
                form.render_in_parent(parent, chunk)?;
            }
            None => {
                parent.render_widget(error_paragraph("", "No secret selected"), chunk);
            }
        }
        Ok(())
    }
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        let code = event.code;
        Ok(Propagate)
    }
}
pub fn error_paragraph<'a>(title: &'a str, content: &'a str) -> Paragraph<'a> {
    Paragraph::new(content)
        .style(Style::default().fg(ui::color_light()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::DarkGray))
                .title(title)
                .border_type(BorderType::Plain),
        )
}
