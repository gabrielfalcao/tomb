#![allow(unused_imports)]
#![allow(unused_variables)]

use super::super::geometry::*;
use super::super::ui;

use crate::app::form::{Form, TextField};
use crate::app::TombConfig;
use crate::core::version;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct TombConfiguration<'a> {
    tomb_config: TombConfig,
    form: Form,
    focused: bool,
    phantom: PhantomData<&'a Option<()>>,
}

pub fn create_fields(config: TombConfig) -> Vec<SharedField> {
    let mut fields: Vec<SharedField> = Vec::new();
    // pub key_filename: String,
    // pub tomb_filename: String,
    // pub log_filename: String,

    let field_key_filename = TextField::new(
        "key_filename",
        "key_filename",
        config.key_filename.clone(),
        true,
        true,
    );
    let field_tomb_filename = TextField::new(
        "tomb_filename",
        "tomb_filename",
        config.tomb_filename.clone(),
        true,
        true,
    );
    let field_log_filename = TextField::new(
        "log_filename",
        "log_filename",
        config.log_filename.clone(),
        true,
        true,
    );
    fields.push(Rc::new(RefCell::new(field_key_filename)));
    fields.push(Rc::new(RefCell::new(field_tomb_filename)));
    fields.push(Rc::new(RefCell::new(field_log_filename)));
    fields
}
impl<'a> TombConfiguration<'a> {
    pub fn new(tomb_config: TombConfig) -> TombConfiguration<'a> {
        let form = Form::new(
            "TombConfiguration",
            Some("Tomb TombConfiguration".to_string()),
            create_fields(tomb_config.clone()),
        );
        TombConfiguration {
            tomb_config,
            form,
            focused: false,
            phantom: PhantomData,
        }
    }
    pub fn border_style(&self) -> Color {
        match self.focused {
            true => ui::color_light(),
            false => ui::color_default(),
        }
    }
}

impl Component for TombConfiguration<'_> {
    fn name(&self) -> &str {
        "Tomb TombConfiguration"
    }
    fn id(&self) -> String {
        String::from("TombConfiguration")
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(ui::block_style().fg(self.border_style()))
            .border_type(BorderType::Thick)
            .title(String::from("Locations"));

        rect.render_widget(modal.clone(), chunk);
        self.form.render_in_parent(rect, modal.inner(chunk))
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Down => {
                self.form.tab(false);
                Ok(Propagate)
            }
            KeyCode::Up => {
                self.form.tab(true);
                Ok(Propagate)
            }
            _ => self.form.process_keyboard(event, terminal, context, router),
        }
    }
}
impl Route for TombConfiguration<'_> {}
impl Focusable for TombConfiguration<'_> {
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
        self.form.blur();
    }
}
