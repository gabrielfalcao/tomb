pub mod components;
pub mod geometry;
pub mod logging;
pub mod routes;
pub mod state;

use crate::ironpunk;
pub use components::{menu::Menu, modal::Modal, searchbox::SearchBox};

pub use {application::*, geometry::*, logging::*, routes::*, state::*};

use crate::aes256cbc::{Config as AesConfig, Key};
use crate::tomb::{AES256Secret, AES256Tomb};

use std::{cell::RefCell, rc::Rc};

pub fn start(
    tomb: AES256Tomb,
    key: Key,
    aes_config: AesConfig,
    tick_interval: u64,
) -> Result<(), ironpunk::SharedError> {
    let mut router = ironpunk::SharedRouter::new();

    let app = Rc::new(RefCell::new(Application::new(
        key.clone(),
        tomb.clone(),
        aes_config.clone(),
    )));

    router.add(
        "/about",
        Rc::new(RefCell::new(About::new(aes_config.clone()))),
    );
    router.add(
        "/delete/:key",
        Rc::new(RefCell::new(DeleteSecret::new(
            key.clone(),
            tomb.clone(),
            aes_config.clone(),
        ))),
    );
    router.add("/:filter", app.clone());
    router.add("/", app);
    ironpunk::start(router, tick_interval)
}
