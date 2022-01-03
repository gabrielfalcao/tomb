use super::super::components::menu::Menu;
use super::super::geometry::*;
use super::super::ui;

use crate::app::TombConfig;

use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    text::Text,
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

const COMPONENT_NAME: &'static str = "Help";

#[allow(dead_code)]
#[derive(Clone)]
pub struct Help<'a> {
    tomb_config: TombConfig,
    menu: Menu,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> Help<'a> {
    pub fn new(tomb_config: TombConfig) -> Help<'a> {
        Help {
            tomb_config,
            menu: Menu::default(COMPONENT_NAME),
            phantom: PhantomData,
        }
    }
}

impl Component for Help<'_> {
    fn name(&self) -> &str {
        COMPONENT_NAME
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

        let block = Block::default()
            .borders(Borders::ALL)
            .style(ui::default_style().fg(ui::color_default()))
            .title("<press (Esc) to dismiss>")
            .border_type(BorderType::Plain);

        let help = Paragraph::new(Text::from(
            r#"
Keyboard Shortcuts:
~~~~~~~~~~~~~~~~~~~
  'f' to filter secrets by path using glob patterns
  'tab' focus on secret metadata
  'esc' focus on list of secrets
  'up' and 'down' arrows browse secrets or their metadata fields

  Secrets
  ~~~~~~~
    't' toggle visibility
    'r' reveal
    'c' copy to clipboard

  Screens
  ~~~~~~~
    'H' or '?' show this help screen
    'C' show configuration screen
    'A' show about screen
    'left' and 'right' move between screens
"#,
        ))
        .style(ui::default_style().fg(ui::color_light()))
        .alignment(Alignment::Left)
        .block(block);
        rect.render_widget(help, chunk);
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
impl Route for Help<'_> {}
