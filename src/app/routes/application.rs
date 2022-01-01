pub use super::super::components::{menu::Menu, modal::Modal, searchbox::SearchBox};
use super::super::geometry::*;
use super::super::log_error;
pub use super::super::state::*;
use super::super::ui;
use crate::ioutils::log_to_file;
use chrono::prelude::*;

use crate::ironpunk::*;

extern crate clipboard;
use super::super::{AES256Secret, AES256Tomb, TombConfig};
use crate::aes256cbc::{Config as AesConfig, Key};

use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent};
use mac_notification_sys::*;
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Cell, List, ListItem, Row, Table, Wrap},
    Terminal,
};

const DEFAULT_STATUS: &'static str =
    "'f' to filter / 't' toggle visibility / 'r' reveal / 'c' copy to clipboard";

pub fn log(message: String) {
    log_to_file("application.log", message).unwrap()
}
#[allow(dead_code)]
pub struct Application<'a> {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    tomb_config: TombConfig,
    phantom: PhantomData<&'a List<'a>>,
    started_at: DateTime<Utc>,
    pub label: String,
    pub text: String,
    pub error: Option<String>,
    pub visible: bool,
    pub pin_visible: bool,
    pub menu: Menu,
    pub searchbox: SearchBox,
    pub scroll: u16,
    pub items: StatefulList,
}

impl<'a> Application<'a> {
    pub fn new(
        key: Key,
        tomb: AES256Tomb,
        tomb_config: TombConfig,
        aes_config: AesConfig,
    ) -> Application<'a> {
        log_error(format!("tomb opened"));
        Application {
            key,
            tomb,
            aes_config,
            tomb_config,
            menu: Menu::default("Secrets"),
            searchbox: SearchBox::new("*"),
            started_at: Utc::now(),
            text: String::from("Up/Down browse secrets / 'f' search secrets"),
            label: String::from("Keyboard Shortcuts"),
            visible: false,
            pin_visible: false,
            scroll: 0,
            error: None,
            items: StatefulList::empty(),
            phantom: PhantomData,
        }
    }
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    pub fn show_search(&mut self) {
        self.searchbox.toggle_visible();
    }
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
    pub fn set_pinned(&mut self, pin_visible: bool) {
        self.pin_visible = pin_visible;
    }
    pub fn filter_search(&mut self, pattern: &str) {
        match self.tomb.clone().list(pattern) {
            Ok(items) => {
                self.items.update(items);
            }
            Err(err) => self.error = Some(format!("Search error: {}", err)),
        };
    }

    pub fn log_visibility(&mut self) {
        if self.visible {
            match self.items.current() {
                Some(secret) => {
                    log_error(format!("Browsing visible secret: {}", secret.path));
                }
                None => {}
            }
        }
    }
    pub fn render_secrets(&mut self) -> Result<(List<'a>, Table<'a>), Error> {
        match self.tomb.reload() {
            // load latest version from disk
            Ok(_) => {}
            Err(e) => return Err(Error::with_message(format!("{}", e))),
        };
        let secrets = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::DarkGray))
            .title("Secret")
            .border_type(BorderType::Plain);
        let items: Vec<_> = self
            .items
            .items
            .iter()
            .map(|secret| {
                ListItem::new(Spans::from(vec![Span::styled(
                    secret.path.clone(),
                    Style::default(),
                )]))
            })
            .collect();

        let pattern = self.searchbox.pattern.clone();
        self.filter_search(&pattern);
        let selected_secret = match self.items.current() {
            Some(secret) => secret,
            None => match self.items.items.len() > 0 {
                true => self.items.items[0].clone(),
                false => {
                    return Err(Error::with_message(format!(
                    "no secrets to list using pattern: {}. Press 'f' to change the search pattern.",
                    pattern
                )))
                }
            },
        };

        let list = List::new(items)
            .block(secrets)
            .highlight_style(Style::default().bg(ui::color_default()).fg(Color::White));

        let secret = selected_secret.clone();
        let secret_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(format!(
                "{}",
                selected_secret
                    .digest
                    .iter()
                    .map(|b| format!("{:02x}", *b))
                    .collect::<Vec::<_>>()
                    .join("")
            ))),
            Cell::from(Span::raw(selected_secret.path)),
            Cell::from(Span::raw(match self.visible {
                true => match self.get_plaintext(&secret) {
                    Ok(plaintext) => plaintext,
                    Err(err) => format!("{}", err),
                },
                false => secret.value,
            })),
            Cell::from(Span::raw(
                chrono_humanize::HumanTime::from(selected_secret.updated_at).to_string(),
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
        ]);

        Ok((list, secret_detail))
    }
    pub fn search_visible(&self) -> bool {
        self.searchbox.visible
    }
    pub fn set_text(&mut self, text: &str) {
        self.text = String::from(text);
    }
    #[allow(dead_code)]
    pub fn set_error(&mut self, error: String) {
        self.error = Some(error.clone());
    }
    pub fn set_label(&mut self, label: &str) {
        self.label = String::from(label);
    }
    pub fn selected_secret(&mut self) -> Result<AES256Secret, Error> {
        match self.items.current() {
            Some(secret) => Ok(secret),
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
    pub fn reset_statusbar(&mut self) {
        if !self.pin_visible {
            self.set_visible(false);
        }
        match self.selected_secret() {
            Ok(_) => {
                let label = format!("Keyboard Shortcuts");
                self.set_label(label.as_str());
                self.set_text(DEFAULT_STATUS);
            }
            Err(err) => {
                let error = format!("{}", err);
                self.set_label("Error");
                self.set_text(&error);
            }
        }
        self.log_visibility();
    }
}

impl Component for Application<'_> {
    fn name(&self) -> &str {
        "Application"
    }
    fn id(&self) -> String {
        String::from("Application")
    }
    fn tick(
        &mut self,
        _terminal: &mut Terminal<Backend>,
        _context: SharedContext,
        _router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        Ok(Propagate)
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        let code = event.code;
        match self.search_visible() {
            true => {
                return self.searchbox.process_keyboard(
                    event,
                    terminal,
                    context.clone(),
                    router.clone(),
                );
            }
            false => {
                self.menu
                    .process_keyboard(event, terminal, context.clone(), router.clone())?;
                match code {
                    KeyCode::Char('q') => {
                        log_error(format!("tomb closed"));
                        Ok(Quit)
                    }
                    KeyCode::Char('d') => match self.items.current() {
                        Some(secret) => {
                            let path = format!("/delete/{}", secret.key());
                            context.borrow_mut().goto(&path);
                            Ok(Propagate)
                        }
                        None => Err(Error::with_message(format!(
                            "cannot delete: no secret selected"
                        ))),
                    },
                    KeyCode::Char('a') => {
                        context.borrow_mut().goto("/about");
                        Ok(Refresh)
                    }
                    KeyCode::Char('s') => {
                        context.borrow_mut().goto("/");
                        Ok(Refresh)
                    }
                    KeyCode::Char('f') => {
                        self.show_search();
                        Ok(Refresh)
                    }
                    KeyCode::Char('r') => {
                        match self.selected_secret_string() {
                            Ok(plaintext) => {
                                self.reset_statusbar();
                                self.set_visible(true);
                                self.set_pinned(false);
                                self.set_text(&format!("Secret: {}", plaintext));
                            }
                            Err(error) => {
                                log_error(format!("cannot reveal secret: {}", error));
                            }
                        };
                        Ok(Refresh)
                    }
                    KeyCode::Char('t') => {
                        match self.selected_secret_string() {
                            Ok(plaintext) => {
                                self.set_pinned(true);
                                self.toggle_visible();
                                self.set_text(match self.visible {
                                    true => "Secrets visible. (Press 't' again to toggle)",
                                    false => "Secrets hidden. (Press 't' again to toggle)",
                                });
                            }
                            Err(error) => {
                                log_error(format!("cannot toggle visibility: {}", error));
                            }
                        };
                        log_error(format!("toggle visible: {:?}", self.visible));

                        Ok(Refresh)
                    }
                    KeyCode::Up => {
                        self.items.previous();
                        self.reset_statusbar();
                        Ok(Propagate)
                    }
                    KeyCode::Down => {
                        self.items.next();
                        self.reset_statusbar();
                        Ok(Propagate)
                    }
                    KeyCode::Esc => {
                        // TODO: context.error.clear()
                        Ok(Propagate)
                    }
                    KeyCode::Char('c') | KeyCode::Enter => match self.items.current() {
                        Some(secret) => match self.selected_secret_string() {
                            Ok(plaintext) => {
                                let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                                ctx.set_contents(plaintext).unwrap();
                                log_error(format!("copied secret to clipboard: {:?}", secret.path));
                                let text = format!("{:?} copied to clipboard", secret.path);
                                self.set_text(&text);
                                send_notification(
                                    format!("Secret {}", secret.path).as_str(),
                                    &Some("copied to clipboard"),
                                    "",
                                    &Some("Glass"),
                                )
                                .unwrap();
                                Ok(Propagate)
                            }
                            Err(error) => {
                                context
                                    .borrow_mut()
                                    .error
                                    .set_error(Error::with_message(format!("{}", error)));
                                Ok(Propagate)
                            }
                        },
                        None => Ok(Propagate),
                    },
                    _ => Ok(Propagate),
                }
            }
        }
    }
}
impl Route for Application<'_> {
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        _router: SharedRouter,
    ) -> Result<(), Error> {
        terminal.draw(|rect| {
            let (top, body, footer) = vertical_stack(rect.size());
            let (_top_left, top_right) = body_sides(top);
            // let top_right = overlay_position(body);
            let (sidebar, detail) = body_sides(body);
            let location = context.borrow().location.clone();
            match self.render_secrets() {
                Ok((left, right)) => {
                    rect.render_stateful_widget(left, sidebar, &mut self.items.state);
                    rect.render_widget(right, detail);
                }
                Err(error) => {
                    let error =
                        error_text("Application Error", "Uncaught exception:", &error.message);
                    rect.render_widget(error, get_modal_rect(body));
                }
            };
            let (footer_title, footer_label) = match self.error.clone() {
                Some(error) => (error.clone(), self.text.clone()),
                None => (self.label.clone(), self.text.clone()),
            };
            let status_bar = status_paragraph(&footer_title, &footer_label);
            // select menu item based on current route
            self.menu.select_by_location(location);
            self.menu.render_in_parent(rect, top).unwrap();
            if self.search_visible() {
                self.searchbox.render_in_parent(rect, top_right).unwrap();
            }
            rect.render_widget(status_bar, footer);
        })?;
        Ok(())
    }
}

pub fn status_paragraph<'a>(title: &'a str, content: &'a str) -> Paragraph<'a> {
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
pub fn error_text<'a>(label: &'a str, title: &'a str, error: &'a str) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw(title)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            error,
            //console::strip_ansi_codes(self.error.message.as_str()).borrow(),
        )]),
    ])
    .alignment(Alignment::Center)
    .wrap(Wrap { trim: false })
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(ui::color_default()).fg(Color::White))
            .title(label)
            .border_type(BorderType::Plain),
    )
}
pub fn overlay_position(size: Rect) -> Rect {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Max(3), Constraint::Percentage(80)].as_ref())
        .split(size);
    chunks[0]
}
