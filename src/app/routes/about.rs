#![allow(unused_imports)]
#![allow(unused_variables)]

use super::super::components::menu::Menu;
use super::super::geometry::*;
use super::super::ui;

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

const COMPONENT_NAME: &'static str = "About";

#[allow(dead_code)]
#[derive(Clone)]
pub struct About<'a> {
    tomb_config: TombConfig,
    menu: Menu,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> About<'a> {
    pub fn new(tomb_config: TombConfig) -> About<'a> {
        About {
            tomb_config,
            menu: Menu::default(COMPONENT_NAME),
            phantom: PhantomData,
        }
    }
}

impl Component for About<'_> {
    fn name(&self) -> &str {
        COMPONENT_NAME
    }
    fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.menu.tick(terminal, context, router)
    }

    fn id(&self) -> String {
        String::from(COMPONENT_NAME)
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let (header, chunk, footer) = vertical_stack(chunk);

        self.menu.render_in_parent(rect, header)?;
        self.menu.render_in_parent(rect, footer)?;
        let version = format!("Version {}", version());
        let block = Block::default()
            .borders(Borders::ALL)
            .style(ui::default_style().fg(ui::color_default()))
            .title("<press (Esc) to dismiss>")
            .border_type(BorderType::Plain);

        let containers = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                    Constraint::Percentage(33),
                ]
                .as_ref(),
            )
            .split(chunk);

        let top = Paragraph::new(vec![
            Spans::from(vec![Span::raw("️⚰Tomb - Password Manager")]),
            Spans::from(vec![Span::raw(&version)]),
        ])
        .style(ui::default_style().fg(ui::color_light()))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::NONE)
                .style(ui::default_style().fg(ui::color_light())),
        );
        let middle = Paragraph::new(vec![
            Spans::from(vec![Span::raw("powered by")]),
            Spans::from(vec![Span::styled(
                "AES-256-CBC",
                ui::default_style().fg(ui::color_default()),
            )]),
            Spans::from(vec![Span::raw("encryption")]),
        ])
        .style(ui::default_style().fg(ui::color_default()))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
        let bottom = Paragraph::new(vec![
            Spans::from(vec![Span::raw("Created by: Gabriel Falcão")]),
            Spans::from(vec![Span::raw("twitter: @gabrielfalcao")]),
            // Spans::from(vec![Span::raw("https://github.com/gabrielfalcao/tomb")]),
        ])
        .style(ui::default_style().fg(ui::color_light()))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::NONE));

        rect.render_widget(block, chunk);
        rect.render_widget(top, get_padded_rect(containers[0], 2));
        rect.render_widget(middle, get_padded_rect(containers[1], 1));
        rect.render_widget(bottom, get_padded_rect(containers[2], 1));
        Ok(())
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.menu.process_keyboard(event, terminal, context, router)
    }
}
impl Route for About<'_> {}
