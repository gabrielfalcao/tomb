use super::super::components::confirmation::{
    highlight_style, paragraph_style, ConfirmationDialog, ConfirmationOption,
};
use crate::aes256cbc::Config as AesConfig;
use crate::aes256cbc::Key;

// use crate::config::YamlFile;
use crate::app::TombConfig;
use crate::ironpunk::*;
use crate::tomb::{default_tomb_filename, AES256Secret, AES256Tomb};

use super::super::logging::log_error;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{backend::CrosstermBackend, Terminal};

#[allow(dead_code)]
#[derive(Clone)]
pub struct DeleteSecret<'a> {
    key: Key,
    secret_path: Option<String>,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    tomb_config: TombConfig,
    phantom: PhantomData<&'a Option<()>>,
    dialog: ConfirmationDialog<'a>,
    tomb_filepath: String,
}

impl<'a> DeleteSecret<'a> {
    pub fn new(
        key: Key,
        tomb: AES256Tomb,
        tomb_config: TombConfig,
        aes_config: AesConfig,
    ) -> DeleteSecret<'a> {
        DeleteSecret {
            key,
            tomb,
            aes_config,
            tomb_config,
            secret_path: None,
            phantom: PhantomData,
            dialog: ConfirmationDialog::new(None),
            tomb_filepath: default_tomb_filename(),
        }
    }
    pub fn get_secret(
        &mut self,
        context: SharedContext,
        router: SharedRouter,
    ) -> Option<AES256Secret> {
        let path = context.borrow().location.clone();
        match router.recognize(path.as_str()) {
            Ok(matched) => {
                let params = matched.params();
                match params.find("key") {
                    Some(key) => match self.tomb.data.get(key) {
                        Some(secret) => Some(secret.clone()),
                        None => None,
                    },
                    None => None,
                }
            }
            Err(err) => {
                log_error(format!("{}", err));
                None
            }
        }
    }
    fn delete_secret(
        &mut self,
        context: SharedContext,
        secret: AES256Secret,
    ) -> Result<LoopEvent, Error> {
        let path = secret.path.clone();

        match self.tomb.delete_secret(&path) {
            Ok(_) => match self.tomb.save() {
                Ok(_) => {
                    log_error(format!("deleted secret: {}", path));
                    context.borrow_mut().goto("/");
                    Ok(Propagate)
                }
                Err(err) => {
                    log_error(format!("error deleting secret {}: {}", path, err));
                    Ok(Quit)
                }
            },
            Err(err) => {
                log_error(format!("error deleting secret {}: {}", path, err));
                Ok(Quit)
            }
        }
    }
}

impl Component for DeleteSecret<'_> {
    fn name(&self) -> &str {
        "DeleteSecret"
    }
    fn id(&self) -> String {
        match self.secret_path.clone() {
            None => panic!("DeleteSecret route did not receive the secret_path"),
            Some(path) => format!("DeleteSecret:{}", path),
        }
    }
    fn render_in_parent(
        &mut self,
        rect: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        self.dialog.render_in_parent(rect, chunk)
    }

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        self.dialog
            .process_keyboard(event, terminal, context.clone(), router.clone())?;
        let path = context.borrow().location.clone();
        match event.code {
            KeyCode::Esc => {
                context.borrow_mut().goback();
                Ok(Propagate)
            }
            KeyCode::Enter => {
                let secret = match self.get_secret(context.clone(), router.clone()) {
                    Some(secret) => secret,
                    None => {
                        return Err(Error::with_message(format!(
                            "failed to retrieve secret: {}",
                            path
                        )));
                    }
                };
                match self.dialog.choice() {
                    ConfirmationOption::Yes => {
                        match self.get_secret(context.clone(), router.clone()) {
                            Some(secret) => self.delete_secret(context.clone(), secret),
                            None => Ok(Propagate),
                        }
                    }
                    ConfirmationOption::No => {
                        log_error(format!("canceled deletion of secret {}", path));
                        context.borrow_mut().goback();
                        Ok(Refresh)
                    }
                }
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
impl Route for DeleteSecret<'_> {
    fn render(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        match self.get_secret(context.clone(), router.clone()) {
            Some(secret) => {
                self.dialog.set_question(Some(vec![
                    Spans::from(vec![Span::styled(
                        "Are you sure you want to delete the secret",
                        paragraph_style(),
                    )]),
                    Spans::from(vec![Span::styled(secret.path.clone(), highlight_style())]),
                    Spans::from(vec![Span::styled("?", paragraph_style())]),
                ]))?;
            }
            None => {
                self.dialog
                    .set_question(Some(vec![Spans::from(vec![Span::styled(
                        "could not retrieve secret",
                        paragraph_style(),
                    )])]))?;
            }
        };

        terminal.draw(|parent| {
            let chunk = parent.size();
            match self.render_in_parent(parent, chunk) {
                Ok(_) => (),
                Err(err) => {
                    log(format!(
                        "error rendering component {}: {}",
                        self.name(),
                        err
                    ));
                }
            }
        })?;

        Ok(())
    }
}
