#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub use super::super::components::{menu::Menu, modal::Modal, searchbox::SearchBox};
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
    fn create_widget(&mut self, secret: AES256Secret) -> Table<'a> {
        Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(format!(
                "{}",
                secret
                    .digest
                    .iter()
                    .map(|b| format!("{:02x}", *b))
                    .collect::<Vec::<_>>()
                    .join("")
            ))),
            Cell::from(Span::raw(secret.path.clone())),
            Cell::from(Span::raw(match self.visible {
                true => match self.get_plaintext(&secret.clone()) {
                    Ok(plaintext) => plaintext,
                    Err(err) => format!("{}", err),
                },
                false => secret.value,
            })),
            Cell::from(Span::raw(
                chrono_humanize::HumanTime::from(secret.updated_at).to_string(),
            )),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "digest",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "name",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "value",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "updated at",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(ui::color_default()))
                .title("Metadata")
                .border_type(BorderType::Plain),
        )
        .widths(&[
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(20),
        ])
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
                let widget = self.create_widget(secret);
                parent.render_widget(widget, chunk);
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
