use super::super::geometry::*;
use super::super::ui;
use super::super::{AES256Tomb, TombConfig};
use crate::aes256cbc::Key;

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

#[allow(dead_code)]
#[derive(Clone)]
pub struct EditSecret<'a> {
    key: Key,
    tomb: AES256Tomb,
    tomb_config: TombConfig,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> EditSecret<'a> {
    pub fn new(key: Key, tomb: AES256Tomb, tomb_config: TombConfig) -> EditSecret<'a> {
        EditSecret {
            key,
            tomb,
            tomb_config,
            phantom: PhantomData,
        }
    }
}

impl Component for EditSecret<'_> {
    fn name(&self) -> &str {
        "EditSecret"
    }
    fn id(&self) -> String {
        String::from("EditSecret")
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let (_header, chunk, _footer) = vertical_stack(chunk);

        let block = Block::default()
            .borders(Borders::ALL)
            .style(ui::default_style().fg(ui::color_default()))
            .title("<press (Esc) to dismiss>")
            .border_type(BorderType::Plain);

        let help = Paragraph::new(Text::from(
            r#"
Keyboard Shortcuts:
~~~~~~~~~~~~~~~~~~~

    'tab' focus on secret metadata
    'esc' focus on list of secrets
    'f' to filter
    't' toggle visibility
    'r' reveal
    'c' copy to clipboard
    '?' or 'h' show this help
    'a' about
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
        _router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Esc => {
                context.borrow_mut().goto("/");
                Ok(Refresh)
            }
            KeyCode::Char('q') => Ok(Quit),
            KeyCode::Left => {
                context.borrow_mut().goback();
                Ok(Refresh)
            }
            KeyCode::Right => {
                context.borrow_mut().goto("/about");
                Ok(Refresh)
            }
            _ => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('q') {
                    return Ok(Quit);
                }
                Ok(Propagate)
            }
        }
    }
}
impl Route for EditSecret<'_> {}
