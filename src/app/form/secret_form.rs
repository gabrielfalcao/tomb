#![allow(unused_imports)]
#![allow(unused_variables)]

use super::super::geometry::*;
use super::super::ui;

use crate::app::config::AES256Secret;
use crate::app::form::{Form, RGBColorField};
use crate::app::TombConfig;
use crate::core::version;
use crate::ironpunk::*;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::{io, marker::PhantomData};
use tui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
    Terminal,
};

#[allow(dead_code)]
#[derive(Clone)]
pub struct SecretForm<'a> {
    key: Key,
    tomb: AES256Tomb,
    aes_config: AesConfig,
    tomb_config: TombConfig,
    form: Form,
    focused: bool,
    read_only: bool,
    phantom: PhantomData<&'a Option<()>>,
}

pub fn create_fields(
    key: Key,
    tomb: AES256Tomb,
    tomb_config: TombConfig,
    read_only: bool,
) -> Vec<SharedField> {
    let visible = true;
    let mut fields: Vec<SharedField> = Vec::new();
    let field_name = TextField::new("name", "name", secret.name(), read_only, visible);
    let field_group = TextField::new("group", "group", secret.group(), read_only, visible);
    let field_secret = SecretField::new("secret", "value", read_only, visible, secret, tomb, key);
    let field_digest = TextField::new("digest", "digest", secret.hexdigest(), read_only, visible);
    let field_notes = TextField::new(
        "notes",
        "notes",
        match secret.notes {
            Some(notes) => notes.clone(),
            None => String::from("<none>"),
        },
        read_only,
        visible,
    );
    let field_username = TextField::new(
        "username",
        "username",
        match secret.username {
            Some(username) => username.clone(),
            None => String::from("<none>"),
        },
        read_only,
        visible,
    );
    let field_url = TextField::new(
        "url",
        "url",
        match secret.url {
            Some(url) => url.clone(),
            None => String::from("<none>"),
        },
        read_only,
        visible,
    );
    let field_updated_at = TextField::new(
        "updated-at",
        "updated-at",
        chrono_humanize::HumanTime::from(secret.updated_at).to_string(),
        read_only,
        visible,
    );

    fields.push(Arc::new(RefCell::new(field_name)));
    fields.push(Arc::new(RefCell::new(field_group)));
    fields.push(Arc::new(RefCell::new(field_secret)));
    fields.push(Arc::new(RefCell::new(field_username)));
    fields.push(Arc::new(RefCell::new(field_url)));
    fields.push(Arc::new(RefCell::new(field_updated_at)));
    fields.push(Arc::new(RefCell::new(field_digest)));
    fields.push(Arc::new(RefCell::new(field_notes)));

    fields
}
impl<'a> SecretForm<'a> {
    pub fn new(
        key: Key,
        tomb: AES256Tomb,
        tomb_config: TombConfig,
        read_only: bool,
    ) -> SecretForm<'a> {
        let form = Form::new(
            "SecretForm",
            Some("Tomb SecretForm".to_string()),
            create_fields(key, tomb, tomb_config, read_only),
        );
        SecretForm {
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
    pub fn get_secret(&self) -> AES256Secret {
        let mut result = AES256Secret::empty();
        for field in self.form.fields.iter() {
            let id = field.borrow_mut().get_id();
            match id.as_str() {
                "digest" => {
                    result.digest = field.borrow_mut().get_value().to_vec();
                }
                "path" => {
                    result.path = field.borrow_mut().get_value();
                }
                "value" => {
                    result.value = field.borrow_mut().get_value();
                }
                "url" => {
                    result.url = Some(field.borrow_mut().get_value());
                }
                "username" => {
                    result.username = Some(field.borrow_mut().get_value());
                }
                "notes" => {
                    result.notes = Some(field.borrow_mut().get_value());
                }
                _ => {}
            }
        }
        result
    }
}

impl Component for SecretForm<'_> {
    fn name(&self) -> &str {
        "Tomb SecretForm"
    }
    fn id(&self) -> String {
        String::from("SecretForm")
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
impl Route for SecretForm<'_> {}
impl Focusable for SecretForm<'_> {
    fn tab_index(&self) -> usize {
        1
    }

    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn blur(&mut self) {
        self.form.blur();
        self.focused = false;
    }
}
