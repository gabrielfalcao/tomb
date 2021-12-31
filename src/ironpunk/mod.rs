pub mod base;
pub use base::*;
pub mod geometry;
pub use geometry::*;
pub mod window;
pub use window::*;

use console;
use crossterm::{
    event::{self, Event as CEvent},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::logger;

pub use std::{cell::RefCell, rc::Rc};
use std::{
    io::{self},
    panic,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use tui::{backend::CrosstermBackend, Terminal};

pub fn start(router: SharedRouter, tick_interval: u64) -> Result<(), SharedError> {
    panic::set_hook(Box::new(|e| {
        disable_raw_mode().unwrap_or(());
        reset();
        logger::err::error(format!("{}", e));
    }));

    reset();
    match enable_raw_mode() {
        Ok(_) => {}
        Err(error) => {
            return Err(Box::new(Error::with_message(format!(
                "cannot initialize crossterm: {}",
                error
            ))))
        }
    };

    console::set_colors_enabled(false);
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(tick_interval);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                match event::read().expect("can read events") {
                    CEvent::Key(event) => {
                        tx.send(Event::Input(event)).expect("can send events");
                    }
                    CEvent::Mouse(_event) => {}
                    CEvent::Resize(_width, _height) => {}
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    let mut window = Window::from_routes(router.clone());
    let context = Rc::new(RefCell::new(window.context.clone()));

    loop {
        window.render(&mut terminal, context.clone(), router.clone())?;

        match rx.recv()? {
            Event::Input(event) => {
                if event.modifiers == KeyModifiers::CONTROL && event.code == KeyCode::Char('c') {
                    exit(&mut terminal, 0);
                }

                match window.process_keyboard(event, &mut terminal, context.clone(), router.clone())
                {
                    Ok(Quit) => {
                        quit(&mut terminal);
                    }
                    Ok(Propagate) => continue,
                    Ok(Prevent) => break Ok(()),
                    Ok(Refresh) => {
                        window.render(&mut terminal, context.clone(), router.clone())?;
                    }
                    Err(err) => {
                        log(format!("{}", err));

                        context.borrow_mut().error.set_error(err);
                        window.render(&mut terminal, context.clone(), router.clone())?;
                    }
                };
            }
            Event::Tick => {
                match window.tick(&mut terminal, context.clone(), router.clone()) {
                    Ok(Refresh) => {
                        window.render(&mut terminal, context.clone(), router.clone())?;
                        continue;
                    }
                    Ok(Prevent | Propagate) => continue,
                    Ok(Quit) => {
                        //Ok(return Box::new(quit(&mut terminal))),
                        disable_raw_mode()?;
                        terminal.show_cursor()?;
                        terminal.clear()?;
                        reset();
                        std::process::exit(0);
                    }
                    Err(err) => return Err(Box::new(Error::with_message(format!("{}", err)))),
                };
            }
        };
    }
}
