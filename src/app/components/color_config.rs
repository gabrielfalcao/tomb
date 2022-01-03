#![allow(unused_imports)]
#![allow(unused_variables)]

use super::super::geometry::*;
use super::super::ui;

use crate::app::config::ColorTheme;
use crate::app::form::{Form, RGBColorField};
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
pub struct ColorThemeConfiguration<'a> {
    tomb_config: TombConfig,
    form: Form,
    phantom: PhantomData<&'a Option<()>>,
}

pub fn create_fields(colors: ColorTheme) -> Vec<SharedField> {
    let mut fields: Vec<SharedField> = Vec::new();
    let field_color_default = RGBColorField::new(
        "color_default",
        "default color",
        colors.default,
        false,
        true,
    );
    let field_color_light =
        RGBColorField::new("color_light", "light color", colors.light, false, true);
    let field_color_blurred = RGBColorField::new(
        "color_blurred",
        "blurred color",
        colors.blurred,
        false,
        true,
    );
    let field_color_default_fg = RGBColorField::new(
        "color_default_fg",
        "default_fg color",
        colors.default_fg,
        false,
        true,
    );
    let field_color_default_bg = RGBColorField::new(
        "color_default_bg",
        "default_bg color",
        colors.default_bg,
        false,
        true,
    );
    let field_color_error_fg = RGBColorField::new(
        "color_error_fg",
        "error_fg color",
        colors.error_fg,
        false,
        true,
    );
    let field_color_error_bg = RGBColorField::new(
        "color_error_bg",
        "error_bg color",
        colors.error_bg,
        false,
        true,
    );

    fields.push(Rc::new(RefCell::new(field_color_default)));
    fields.push(Rc::new(RefCell::new(field_color_light)));
    fields.push(Rc::new(RefCell::new(field_color_blurred)));
    fields.push(Rc::new(RefCell::new(field_color_default_fg)));
    fields.push(Rc::new(RefCell::new(field_color_default_bg)));
    fields.push(Rc::new(RefCell::new(field_color_error_fg)));
    fields.push(Rc::new(RefCell::new(field_color_error_bg)));
    fields
}
impl<'a> ColorThemeConfiguration<'a> {
    pub fn new(tomb_config: TombConfig) -> ColorThemeConfiguration<'a> {
        let form = Form::new(
            "ColorThemeConfiguration",
            Some("Tomb ColorThemeConfiguration".to_string()),
            create_fields(tomb_config.colors.clone()),
        );
        ColorThemeConfiguration {
            tomb_config,
            form,
            phantom: PhantomData,
        }
    }
}

impl Component for ColorThemeConfiguration<'_> {
    fn name(&self) -> &str {
        "Tomb ColorThemeConfiguration"
    }
    fn id(&self) -> String {
        String::from("ColorThemeConfiguration")
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let modal = Block::default()
            .borders(Borders::ALL)
            .style(ui::block_style().fg(ui::color_default_fg()))
            .border_type(BorderType::Thick)
            .title(String::from("UI Colors"));

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
impl Route for ColorThemeConfiguration<'_> {}
