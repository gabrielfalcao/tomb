#![allow(unused_imports)]
#![allow(unused_variables)]
use super::super::components::{ColorThemeConfiguration, SharedMenu, TombConfiguration};
use super::super::geometry::*;
use super::super::ui;
use crate::app::form::{Form, RGBColorField};
use crate::app::log_error;
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

#[derive(Eq, PartialEq, Clone)]
enum FocusedComponent {
    Locations,
    UIColors,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Configuration<'a> {
    tomb_config: TombConfig,
    color_configuration: ColorThemeConfiguration<'a>,
    tomb_configuration: TombConfiguration<'a>,
    menu: SharedMenu,
    focused: FocusedComponent,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> Configuration<'a> {
    pub fn new(menu: SharedMenu, tomb_config: TombConfig) -> Configuration<'a> {
        let color_configuration = ColorThemeConfiguration::new(tomb_config.clone());
        let tomb_configuration = TombConfiguration::new(tomb_config.clone());

        Configuration {
            tomb_config,
            menu,
            color_configuration,
            tomb_configuration,
            focused: FocusedComponent::UIColors,
            phantom: PhantomData,
        }
    }
    pub fn switch_focus(&mut self) {
        self.focused = match self.focused {
            FocusedComponent::Locations => FocusedComponent::UIColors,
            FocusedComponent::UIColors => FocusedComponent::Locations,
        }
    }
    pub fn save_config(&mut self) {
        let colors = self.color_configuration.get_color_theme();
        self.tomb_config.set_colors(colors);
        match self.tomb_config.save() {
            Ok(_) => {}
            Err(err) => {
                log_error(format!("failed to save config: {}", err));
            }
        };
    }
}

impl Component for Configuration<'_> {
    fn name(&self) -> &str {
        "Tomb Configuration"
    }
    fn id(&self) -> String {
        String::from("Configuration")
    }
    fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.menu
            .borrow_mut()
            .tick(terminal, context.clone(), router.clone())?;

        match self.focused {
            FocusedComponent::UIColors => self.color_configuration.tick(terminal, context, router),
            FocusedComponent::Locations => self.tomb_configuration.tick(terminal, context, router),
        }
    }

    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let (header, chunk, footer) = vertical_stack(chunk);
        self.menu.borrow_mut().render_in_parent(rect, header)?;
        self.menu.borrow_mut().render_in_parent(rect, footer)?;
        let (left, right) = horizontal_split(chunk);
        match self.focused {
            FocusedComponent::Locations => {
                self.tomb_configuration.focus();
                self.color_configuration.blur();
            }
            FocusedComponent::UIColors => {
                self.color_configuration.focus();
                self.tomb_configuration.blur();
            }
        }
        self.tomb_configuration.render_in_parent(rect, right)?;
        self.color_configuration.render_in_parent(rect, left)
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
            KeyCode::Tab => self.switch_focus(),
            KeyCode::Enter => self.save_config(),
            _ => {}
        }

        match self.menu.borrow_mut().process_keyboard(
            event,
            terminal,
            context.clone(),
            router.clone(),
        ) {
            Ok(Propagate) => match self.focused {
                FocusedComponent::UIColors => self
                    .color_configuration
                    .process_keyboard(event, terminal, context, router),
                FocusedComponent::Locations => self
                    .tomb_configuration
                    .process_keyboard(event, terminal, context, router),
            },
            result => result,
        }
    }
}
impl Route for Configuration<'_> {}
