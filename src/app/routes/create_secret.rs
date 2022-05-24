use super::super::geometry::*;
use super::super::ui;
use super::super::{AES256Tomb, TombConfig};
use crate::aes256cbc::Key;

use crate::app::form::SecretForm;
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
pub struct CreateSecret<'a> {
    key: Key,
    tomb: AES256Tomb,
    tomb_config: TombConfig,
    form: SecretForm<'a>,
    phantom: PhantomData<&'a Option<()>>,
}

impl<'a> CreateSecret<'a> {
    pub fn new(key: Key, tomb: AES256Tomb, tomb_config: TombConfig) -> CreateSecret<'a> {
        let form = SecretForm::new(key.clone(), tomb.clone(), tomb_config.clone(), None, false);
        CreateSecret {
            key,
            tomb,
            tomb_config,
            form,
            phantom: PhantomData,
        }
    }
}

impl Component for CreateSecret<'_> {
    fn name(&self) -> &str {
        "CreateSecret"
    }
    fn id(&self) -> String {
        String::from("CreateSecret")
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

        self.form.render_in_parent(rect, chunk)
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
impl Route for CreateSecret<'_> {}
