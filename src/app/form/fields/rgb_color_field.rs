#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
use crate::app::geometry::*;
use crate::app::log_error;
use crate::app::ui::*;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Rect},
    style::Color,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

#[derive(Debug, Clone)]
pub struct RGBColorField {
    pub title: Option<String>,
    pub value: String,
    pub id: String,
    pub focused: bool,
    pub visible: bool,
    pub read_only: bool,
}

impl RGBColorField {
    pub fn new(
        id: &str,
        title: &str,
        value: String,
        read_only: bool,
        visible: bool,
    ) -> RGBColorField {
        RGBColorField {
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
    pub fn to_color(&mut self) -> Option<Color> {
        let color = self.get_value();
        rgb_to_color(color.as_str())
    }
}

impl Component for RGBColorField {
    fn name(&self) -> &str {
        "RGBColorField"
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let (left, right) = horizontal_split(chunk);

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

        let color_style = block_style()
            .bg(match self.to_color() {
                Some(color) => color,
                None => color_default_bg(),
            })
            .fg(color_default_bg());
        let color = Paragraph::new(match self.to_color() {
            Some(color) => String::new(),
            None => format!("invalid rgb hex"),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(color_style)
                .border_type(BorderType::Thick),
        )
        .style(color_style)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

        parent.render_widget(paragraph, left);
        parent.render_widget(color, right);

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
                Ok(Refresh)
            }
            KeyCode::Esc => {
                self.blur();
                return Ok(Refresh);
            }
            KeyCode::Enter => {
                return Ok(Refresh);
            }
            KeyCode::Char(c) => {
                self.write(c);
                Ok(Refresh)
            }
            _ => Ok(Propagate),
        }
    }
}
impl Focusable for RGBColorField {
    fn tab_index(&self) -> usize {
        0
    }
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn blur(&mut self) {
        self.focused = false;
    }
}

impl Field for RGBColorField {
    fn write(&mut self, c: char) {
        self.value.push(c);
    }
    fn backspace(&mut self) {
        self.value.pop();
    }
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
