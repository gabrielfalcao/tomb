#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use super::super::components::{menu::Menu, modal::Modal, searchbox::SearchBox};
pub use super::super::form::{Form, SecretField, TextField};
use super::super::geometry::*;
pub use super::super::state::*;
use super::super::ui;
use crate::app::log_error;
use chrono::prelude::*;
use std::collections::BTreeMap;

use crate::ironpunk::*;
#[cfg(feature = "osx")]
use mac_notification_sys::*;

extern crate clipboard;
use super::super::{AES256Secret, AES256Tomb, TombConfig};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

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
    form: Form,
    secret_field: SecretField,
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
        let secret_field = SecretField::new(
            "secret-value-field",
            "value",
            true,
            secret.clone(),
            tomb.clone(),
            key.clone(),
        );
        let form = Form::new(
            "secret-details",
            Some(String::from("Details Title")),
            BTreeMap::new(),
        );
        SecretDetails {
            key,
            tomb,
            secret,
            tomb_config,
            visible,
            form,
            secret_field,
            phantom: PhantomData,
        }
    }
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
        self.secret_field.set_visible(visible);
    }
    pub fn get_field(&self, id: &str) -> Option<SharedField> {
        match self.form.fields.clone().get(id) {
            Some(field) => Some(field.clone()),
            None => None,
        }
    }
    pub fn set_secret(&mut self, secret: AES256Secret) {
        self.secret = Some(secret.clone());
        match &mut self.get_field("secret-value-field") {
            Some(field) => match self.get_plaintext(&secret) {
                Ok(plaintext) => {
                    field.borrow_mut().set_value(plaintext.as_str());
                }
                Err(error) => {
                    log_error(format!(
                        "cannot set secret for field {}: {}",
                        field.borrow().get_id(),
                        error
                    ));
                }
            },
            None => {}
        }
    }
    pub fn tab(&mut self, shift: bool) {
        self.form.tab(shift);
    }
    pub fn blur(&mut self) {
        self.form.blur();
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
    fn populate_form(&mut self, secret: AES256Secret) {
        if self.form.fields.len() > 0 {
            // form already populated
            return;
        }
        let digest = secret
            .digest
            .iter()
            .map(|b| format!("{:02x}", *b))
            .collect::<Vec<_>>()
            .join("");

        let field_name = TextField::new("secret-name-field", "name", secret.name(), true);
        let field_group = TextField::new("secret-group-field", "group", secret.group(), true);

        let field_digest =
            TextField::new("secret-digest-field", "digest", format!("{}", digest), true);
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

        self.form.add_field(field_name);
        self.form.add_field(field_group);
        self.form.add_field(self.secret_field.clone());
        self.form.add_field(field_updated_at);
        self.form.add_field(field_digest);
        self.form.add_field(field_notes);
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
                self.populate_form(secret);
                self.form.render_in_parent(parent, chunk)?;
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
        match event.code {
            KeyCode::Down => {
                self.tab(false);
                Ok(Propagate)
            }
            KeyCode::Up => {
                self.tab(true);
                Ok(Propagate)
            }
            KeyCode::Tab => Ok(Prevent),
            _ => Ok(Propagate),
        }
    }
}
pub fn error_paragraph<'a>(title: &'a str, content: &'a str) -> Paragraph<'a> {
    Paragraph::new(content)
        .style(ui::default_style().fg(ui::color_light()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(ui::error_style())
                .title(title)
                .border_type(BorderType::Plain),
        )
}
