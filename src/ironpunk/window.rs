use super::base::*;

use crossterm::event::KeyEvent;

use std::marker::PhantomData;
pub use std::{cell::RefCell, sync::Arc};

use tui::Terminal;
#[allow(dead_code)]
pub struct Window<'a> {
    pub router: SharedRouter,
    pub context: Context<'a>,
    phantom: PhantomData<&'a Context<'a>>,
}

impl<'a> Window<'a> {
    pub fn from_routes(router: SharedRouter) -> Window<'a> {
        Window {
            router,
            phantom: PhantomData,
            context: Context::new("/"),
        }
    }
    pub fn new() -> Window<'a> {
        Window::from_routes(SharedRouter::new())
    }
    #[allow(unused_variables)]
    pub fn tick(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        let path = context.borrow().location.clone();
        match router.recognize(&path) {
            Ok(matched) => {
                // let href = matched.handler.borrow().location.clone();
                match matched
                    .handler()
                    .borrow_mut()
                    .tick(terminal, context.clone(), router.clone())
                {
                    Ok(any) => {
                        return Ok(any);
                    }
                    Err(err) => {
                        context
                            .borrow_mut()
                            .error
                            .set_error(Error::with_message(format!("{}", err)));
                        return Ok(Refresh);
                    }
                };
            }
            Err(error_string) => {
                log(format!("window.tick() route not matched {}", error_string));
                return Ok(Propagate);
            }
        }
    }
}

impl Component for Window<'_> {
    fn name(&self) -> &str {
        "Window"
    }
    fn id(&self) -> String {
        String::from("Window")
    }
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        if context.borrow().error.exists() {
            let result = context.borrow_mut().error.process_keyboard(
                event,
                terminal,
                context.clone(),
                router.clone(),
            )?;
            log(format!(
                "window.process_keyboard() (for error) result: {:?}",
                result
            ));

            match result {
                Quit => {
                    context.borrow_mut().error.clear();
                    context.borrow_mut().goto("/");
                    return Ok(Refresh);
                }
                event => return Ok(event),
            }
        }

        let path = context.borrow().location.clone();
        match router.recognize(&path) {
            Ok(matched) => {
                // let href = matched.handler.borrow().location.clone();
                match matched.handler().borrow_mut().process_keyboard(
                    event,
                    terminal,
                    context.clone(),
                    router.clone(),
                ) {
                    Err(err) => {
                        log(format!(
                            "window.process_keyboard() failed for {}: {}",
                            path, err
                        ));

                        context.borrow_mut().error.set_error(err);
                        return Ok(Refresh);
                    }
                    result => return result,
                }
            }
            Err(error_string) => {
                log(format!(
                    "window.process_keyboard() route not matched {}",
                    error_string
                ));
                return Ok(Propagate);
            }
        }
    }
}
impl Route for Window<'_> {
    fn render(
        &mut self,
        terminal: &mut Terminal<Backend>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<(), Error> {
        let path = context.borrow().location.clone();
        match router.recognize(&path) {
            Ok(matched) => {
                match matched.handler().borrow_mut().render(
                    terminal,
                    context.clone(),
                    router.clone(),
                ) {
                    Ok(_) => return Ok(()),
                    Err(error) => {
                        context.borrow_mut().error.set_error(error);
                    }
                }
            }
            Err(_error_string) => {
                context
                    .borrow_mut()
                    .error
                    .set_error(Error::with_message(format!(
                        "no handler for route: {}",
                        path
                    )));
            }
        }
        let has_error = context.borrow().error.exists();
        if !has_error {
            let (title, message) = (format!("Error 500"), format!("no routes declared"));

            context
                .borrow_mut()
                .error
                .set_error(Error::with_message(message));
            context.borrow_mut().error.set_title(title);
        }
        let result = context
            .borrow_mut()
            .error
            .render(terminal, context.clone(), router.clone());
        result
    }
}
