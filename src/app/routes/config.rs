#![allow(unused_imports)]
#![allow(unused_variables)]
use super::super::components::{ColorThemeConfiguration, SharedMenu};
use super::super::geometry::*;
use super::super::ui;

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
pub struct Configuration<'a> {
    tomb_config: TombConfig,
    color_config: ColorThemeConfiguration<'a>,
    menu: SharedMenu,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> Configuration<'a> {
    pub fn new(menu: SharedMenu, tomb_config: TombConfig) -> Configuration<'a> {
        let color_config = ColorThemeConfiguration::new(tomb_config.clone());

        Configuration {
            tomb_config,
            menu,
            color_config,
            phantom: PhantomData,
        }
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
        self.color_config.tick(terminal, context, router)
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
        self.color_config.render_in_parent(rect, right)
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match self.menu.borrow_mut().process_keyboard(
            event,
            terminal,
            context.clone(),
            router.clone(),
        ) {
            Ok(Propagate) => self
                .color_config
                .process_keyboard(event, terminal, context, router),
            result => result,
        }
    }
}
impl Route for Configuration<'_> {}
