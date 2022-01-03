#![allow(unused_variables)]

use thiserror::Error;

use crate::{ioutils::log_to_file, logger};
pub use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::disable_raw_mode;

use super::geometry::get_modal_rect;
use route_recognizer::Router;
pub use std::{cell::RefCell, rc::Rc};
use std::{
    fmt,
    io::{self},
    marker::PhantomData,
};
pub use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame, Terminal,
};

pub type Backend = CrosstermBackend<io::Stdout>;
pub type SharedError = Box<dyn std::error::Error>;
pub type SharedComponent = Rc<RefCell<dyn Component>>;
pub type SharedFocusable = Rc<RefCell<dyn Focusable>>;
pub type SharedField = Rc<RefCell<dyn Field>>;
pub type SharedRoute = Rc<RefCell<dyn Route>>;
pub type SharedRouter = Router<SharedRoute>;

#[derive(Debug, Error, Clone)]
pub struct Error {
    pub message: String,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Error {
    pub fn with_message(message: String) -> Error {
        Error {
            message: logger::paint::error(format!("{}", message)),
        }
    }
}
impl From<io::Error> for Error {
    fn from(input: io::Error) -> Error {
        Error::with_message(format!("{:?}", input))
    }
}
pub fn reset() {
    println!("\x1bc\x1b[!p\x1b[?3;4l\x1b[4l\x1b>");
}

pub fn exit(terminal: &mut Terminal<Backend>, code: i32) {
    disable_raw_mode().unwrap_or(());
    terminal.show_cursor().unwrap_or(());
    terminal.clear().unwrap_or(());
    reset();
    std::process::exit(code);
}
pub fn quit(terminal: &mut Terminal<Backend>) {
    exit(terminal, 0)
}

pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Debug)]
pub enum LoopEvent {
    Propagate,
    Prevent,
    Refresh,
    Quit,
}
pub fn log(message: String) {
    log_to_file("ironpunk.log", message).unwrap()
}
pub type SharedContext<'a> = Rc<RefCell<Context<'a>>>;

#[derive(Clone)]
pub struct Context<'a> {
    pub location: String,
    pub history: Vec<String>,
    pub error: ErrorRoute,
    phantom: PhantomData<&'a Context<'a>>,
}

impl<'a> Context<'_> {
    pub fn new(location: &str) -> Context<'a> {
        let location = String::from(location);
        Context {
            location: location.clone(),
            phantom: PhantomData,
            history: vec![location],
            error: ErrorRoute::empty(),
        }
    }
    pub fn goto(&mut self, location: &str) {
        let location = String::from(location);
        self.history.push(location.clone());
        self.location = location.clone();
        // log(format!("goto: {}", location));
    }
    pub fn goback(&mut self) {
        if self.history.len() == 1 {
            return;
        }
        match self.history.pop() {
            Some(_) => {
                let location = self.history[self.history.len() - 1].clone();
                self.location = location.clone();
                // log(format!("goback: {}", location));
            }
            None => {}
        }
    }
    pub fn get_location(&self) -> String {
        self.location.clone()
    }
    pub fn get_history(&self) -> Vec<String> {
        self.history.clone()
    }
}

pub use LoopEvent::*;

pub trait Component {
    fn id(&self) -> String;
    fn name(&self) -> &str;
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error>;

    fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        Ok(Refresh)
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let title = format!("<press (Esc) to dismiss>");
        let message = format!(
            "The method render_in_parent() is not implemented for {}",
            self.name()
        );
        let background = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::DarkGray));

        let not_implemented = Paragraph::new(message)
            .style(Style::default().bg(Color::White).fg(Color::Red))
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::Black))
                    .title(title)
                    .border_type(BorderType::Rounded),
            );

        rect.render_widget(background, chunk);
        rect.render_widget(not_implemented, get_modal_rect(chunk));
        Ok(())
    }
}

pub trait Focusable
where
    Self: Component,
{
    fn tab_index(&self) -> usize;
    fn is_focused(&self) -> bool;
    fn focus(&mut self);
    fn blur(&mut self);
}

pub trait Field
where
    Self: Focusable,
{
    fn get_id(&self) -> String;
    fn constraint(&self) -> Constraint;
    fn get_title(&mut self) -> Option<String>;
    fn set_title(&mut self, title: &str);
    fn get_value(&mut self) -> String;
    fn set_value(&mut self, value: &str);
    fn get_visible(&mut self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn write(&mut self, c: char);
    fn backspace(&mut self);
}

pub trait Route
where
    Self: Component,
{
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        terminal.draw(|parent| {
            let chunk = parent.size();
            match self.render_in_parent(parent, chunk) {
                Ok(_) => (),
                Err(err) => {
                    log(format!(
                        "error rendering component {}: {}",
                        self.name(),
                        err
                    ));
                }
            }
        })?;

        Ok(())
    }
}

#[derive(Clone)]
pub struct ErrorRoute {
    error: Option<Error>,
    title: String,
    label: String,
}
impl ErrorRoute {
    pub fn new_with_message(message: String) -> ErrorRoute {
        ErrorRoute {
            label: String::from("Error"),
            title: String::from("Error"),
            error: Some(Error::with_message(message.clone())),
        }
    }
    pub fn empty() -> ErrorRoute {
        ErrorRoute {
            label: String::from("Error"),
            title: String::from("Error"),
            error: None,
        }
    }
    pub fn set_error(&mut self, error: Error) {
        self.error = Some(error.clone());
    }
    pub fn set_title(&mut self, title: String) {
        self.title = title.clone();
    }
    pub fn set_label(&mut self, label: String) {
        self.label = label.clone();
    }
    pub fn clear(&mut self) {
        self.error = None;
    }
    pub fn exists(&self) -> bool {
        match self.error {
            Some(_) => true,
            None => false,
        }
    }
}

impl Route for ErrorRoute {
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        terminal.draw(|parent| {
            match self.render_in_parent(parent, parent.size()) {
                Ok(_) => {}
                Err(err) => {
                    log(format!("error drawing ErrorRoute: {}", err));
                }
            };
        })?;

        match &self.error {
            Some(error) => {}
            None => {}
        };
        Ok(())
    }
}
impl Component for ErrorRoute {
    fn name(&self) -> &str {
        "ErrorRoute"
    }
    fn id(&self) -> String {
        self.title.clone()
    }

    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        match &self.error {
            Some(error) => {
                let paragraph = error_text(&self.label, &self.title, &error.message);
                let chunk = get_modal_rect(parent.size());
                parent.render_widget(paragraph, chunk);
                Ok(())
            }
            None => Err(Error::with_message(format!(
                "ErrorRoute::render_in_parent as called without containing an error to show"
            ))),
        }
    }

    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        __forbidden_context__: SharedContext,
        __forbidden_router__: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        // Remember: The ErrorRoute is contained inside of the SharedContext itself.
        //
        // context usage here is __forbidden__ because it has already been
        // borrow_muted during window.process_keyboard() which is the
        // intended caller of ErrorRoute.process_keyboard()
        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(Quit),
            _ => Ok(Propagate),
        }
    }
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
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::Red).fg(Color::White))
            .title(label)
            .border_type(BorderType::Plain),
    )
}
