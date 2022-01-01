use super::super::ui;
use crate::app::ui;
use crate::ironpunk::*;
use crossterm::event::{KeyCode, KeyEvent};

use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(PartialEq, Clone)]
pub struct SearchBox {
    pub pattern: String,
    pub tmp: String,
    pub visible: bool,
}
impl SearchBox {
    pub fn new(pattern: &str) -> SearchBox {
        let pattern = String::from(pattern);
        SearchBox {
            tmp: pattern.clone(),
            pattern: pattern,
            visible: false,
        }
    }
    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(block_style())
            .title("Search using glob patterns (<Esc> / <Enter>)")
            .border_type(BorderType::Rounded);

        let text = vec![Spans::from(Span::styled(
            self.tmp.clone(),
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

    pub fn set_tmp(&mut self, tmp: &str) {
        self.tmp = String::from(tmp);
    }

    pub fn write(&mut self, c: char) {
        self.tmp.push(c);
    }

    pub fn backspace(&mut self) {
        self.tmp.pop();
    }
}
impl Component for SearchBox {
    fn name(&self) -> &str {
        "SearchBox"
    }
    fn id(&self) -> String {
        self.tmp.clone()
    }
    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        _router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        let code = event.code;
        match event.code {
            KeyCode::Backspace => {
                self.backspace();
                Ok(Propagate)
            }
            KeyCode::Esc => {
                self.hide();
                return Ok(Propagate);
            }
            KeyCode::Enter => {
                if !self.tmp.is_empty() {
                    self.pattern = self.tmp.clone();
                }
                self.hide();
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
pub fn block_style() -> Style {
    ui::default_style()
        .bg(ui::color_default())
        .fg(ui::color_default_fg())
}

pub fn paragraph_style() -> Style {
    ui::default_style().fg(ui::color_default_fg())
}
