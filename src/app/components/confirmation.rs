#![allow(unused_variables)]
#![allow(dead_code)]
use crate::app::ui;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
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
pub enum ConfirmationOption {
    Yes,
    No,
}

use ConfirmationOption::*;

#[derive(Clone)]
pub struct ConfirmationDialog<'a> {
    pub question: Option<Vec<Spans<'a>>>,
    pub selected: ConfirmationOption,
}

impl<'a> ConfirmationDialog<'a> {
    pub fn new(question: Option<Vec<Spans<'a>>>) -> ConfirmationDialog<'a> {
        ConfirmationDialog {
            question: question,
            selected: No,
        }
    }
    pub fn toggle_selected(&mut self) {
        self.selected = match self.selected {
            ConfirmationOption::No => Yes,
            ConfirmationOption::Yes => No,
        }
    }
    pub fn execute(&mut self) -> Result<LoopEvent, Error> {
        Ok(Propagate)
    }
    pub fn choice(&self) -> ConfirmationOption {
        self.selected.clone()
    }
    pub fn set_question(&mut self, question: Option<Vec<Spans<'a>>>) -> Result<LoopEvent, Error> {
        self.question = question;
        Ok(Propagate)
    }
}

impl<'a> Component for ConfirmationDialog<'a> {
    fn name(&self) -> &str {
        "ConfirmationDialog"
    }
    fn id(&self) -> String {
        String::from("ConfirmationDialog")
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunk = get_modal_rect(chunk);
        let confirmation = Block::default()
            .borders(Borders::ALL)
            .style(block_style())
            .title(format!("Delete Secret"))
            .border_type(BorderType::Thick);

        let (top, bottom) = vertical_split(chunk);

        let question = match self.question.clone() {
            Some(question) => question.clone(),
            None => {
                return Err(Error::with_message(format!(
                    "set_question was never called for ConfirmationDialog"
                )))
            }
        };
        let question = Paragraph::new(question)
            .block(confirmation)
            .style(paragraph_style())
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let button_yes = Paragraph::new(vec![Spans::from(Span::styled(
            format!("Yes, delete"),
            match self.selected {
                Yes => ui::default_style()
                    .bg(Color::LightGreen)
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
                No => ui::default_style().bg(Color::Green).fg(Color::White),
            },
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(match self.selected {
                    Yes => ui::default_style().bg(Color::LightGreen).fg(Color::White),
                    No => ui::default_style().bg(Color::Green).fg(Color::White),
                }),
        )
        .alignment(Alignment::Center);
        let button_no = Paragraph::new(vec![Spans::from(Span::styled(
            format!("No, cancel"),
            match self.selected {
                No => ui::default_style()
                    .bg(Color::LightRed)
                    .fg(Color::White)
                    .add_modifier(Modifier::UNDERLINED),
                Yes => ui::default_style().bg(Color::Red).fg(Color::White),
            },
        ))])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(match self.selected {
                    No => ui::default_style().bg(Color::LightRed).fg(Color::White),
                    Yes => ui::default_style().bg(Color::Red).fg(Color::White),
                }),
        )
        .alignment(Alignment::Center);

        let (left, right) = horizontal_split(bottom);
        parent.render_widget(question, top);
        parent.render_widget(button_yes, left);
        parent.render_widget(button_no, right);

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
            KeyCode::Tab | KeyCode::Left | KeyCode::Right => {
                self.toggle_selected();
                Ok(Propagate)
            }
            KeyCode::Backspace => Ok(Propagate),
            KeyCode::Esc => Ok(Propagate),
            KeyCode::Enter => self.execute(),
            KeyCode::Char(c) => Ok(Refresh),
            _ => Ok(Propagate),
        }
    }
}

pub fn vertical_split(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(size);

    let top = chunks[0];
    let bottom = chunks[1];

    (top, bottom)
}

pub fn horizontal_split(size: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .margin(1)
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    let left = chunks[0];
    let right = chunks[1];

    (left, right)
}
pub fn paragraph_style() -> Style {
    ui::default_style().bg(Color::White).fg(Color::Black)
}
pub fn highlight_style() -> Style {
    ui::default_style()
        .fg(Color::Red)
        .add_modifier(Modifier::UNDERLINED)
}

pub fn block_style() -> Style {
    ui::default_style().bg(Color::White).fg(Color::Black)
}
