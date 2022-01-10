#![allow(unused_variables)]
#![allow(dead_code)]

use crate::app::log_error;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent};
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    Frame, Terminal,
};

#[derive(Clone)]
pub struct Form {
    pub id: String,
    pub title: Option<String>,
    pub fields: Vec<SharedField>,
    pub selected_index: Option<usize>,
}
/// Form with editable content
impl Form {
    pub fn new(id: &str, title: Option<String>, fields: Vec<SharedField>) -> Form {
        Form {
            id: String::from(id),
            title,
            fields,
            selected_index: None,
        }
    }
    pub fn purge_fields(&mut self) {
        self.fields = Vec::new();
    }
    pub fn add_field<T: 'static>(&mut self, field: T)
    where
        T: Field,
    {
        self.fields.push(Arc::new(RefCell::new(field)));
    }
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(String::from(title));
    }
    pub fn remove_title(&mut self) {
        self.title = None;
    }
    pub fn tab(&mut self, shift: bool) {
        let total_fields = self.fields.len();
        if total_fields == 0 {
            return;
        }
        match self.selected_index.clone() {
            Some(index) => {
                let new_index = if shift {
                    if index == 0 {
                        0
                    } else {
                        index - 1
                    }
                } else {
                    index + 1
                };
                self.selected_index = Some(if new_index > 0 {
                    new_index % total_fields
                } else {
                    0
                });
            }
            None => {
                log_error(format!("selected form field: {:?}", 0));
                self.selected_index = Some(if shift { self.fields.len() - 1 } else { 0 });
            }
        };
    }
    pub fn focused_field(&mut self) -> Option<(String, SharedField)> {
        for field in self.fields.iter_mut() {
            if field.borrow().is_focused() {
                let title = match field.borrow_mut().get_title() {
                    Some(title) => title.clone(),
                    None => String::from("field"),
                };
                return Some((title, field.clone()));
            }
        }
        None
    }

    pub fn blur(&mut self) {
        for field in self.fields.iter_mut() {
            field.borrow_mut().blur();
        }
        self.selected_index = None;
    }

    pub fn field_constraints(&self) -> Vec<Constraint> {
        let mut result = Vec::new();
        for field in self.fields.iter() {
            result.push(field.borrow().constraint());
        }
        result
    }
}

impl Component for Form {
    fn name(&self) -> &str {
        "Form"
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let chunks = vertical_stack(chunk, self.field_constraints());

        for (i, field) in self.fields.iter_mut().enumerate() {
            let chunk = chunks[i];
            match self.selected_index.clone() {
                Some(index) => {
                    if i == index {
                        field.borrow_mut().focus();
                    } else {
                        field.borrow_mut().blur();
                    }
                }
                None => {
                    field.borrow_mut().blur();
                }
            }
            field.borrow_mut().render_in_parent(parent, chunk)?;
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
            KeyCode::Esc => {
                self.blur();
                return Ok(Refresh);
            }
            KeyCode::Tab => {
                return Ok(Propagate);
            }
            // forward keyboard event to focused_field
            _ => match self.focused_field() {
                Some((title, field)) => {
                    // log_error(format!(
                    //     "forwarding keyboard event {:?} to field {}",
                    //     event, title,
                    // ));

                    field
                        .borrow_mut()
                        .process_keyboard(event, terminal, context, router)
                }
                None => Ok(Propagate),
            },
        }
    }
}

pub fn vertical_stack(size: Rect, constraints: Vec<Constraint>) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints.as_ref())
        .split(size)
}
