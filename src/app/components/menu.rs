#![allow(unused_variables)]

use super::super::ui;
use crate::app::log_error;
use crate::ironpunk::*;
use crossterm::event::{KeyCode, KeyEvent};
use route_recognizer::Router;
use std::{collections::BTreeMap, io};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::Modifier,
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame, Terminal,
};

#[derive(PartialEq, Clone, Debug)]
pub struct MenuItem {
    pub label: String,
    pub route_path: String,
    pub code: KeyCode,
}
pub type SharedMenu = Rc<RefCell<Menu>>;
impl MenuItem {
    pub fn new(label: String, code: KeyCode, route_path: String) -> MenuItem {
        MenuItem {
            label,
            code,
            route_path,
        }
    }
}

#[derive(Clone)]
pub struct Menu {
    pub cid: String,
    pub selected: Option<usize>,
    pub labels: Vec<String>,
    pub router: Router<MenuItem>,
    pub items: BTreeMap<String, MenuItem>,
    pub error: Option<String>,
}
impl Menu {
    pub fn empty() -> Menu {
        Menu {
            cid: String::from("main-menu"),
            selected: None,
            labels: Vec::new(),
            items: BTreeMap::new(),
            router: Router::new(),
            error: None,
        }
    }
    pub fn default() -> Menu {
        let mut menu = Menu::empty();
        menu.add_item("Secrets", KeyCode::Char('S'), "/");
        menu.add_item("Help", KeyCode::Char('H'), "/help");
        menu.add_item("Configuration", KeyCode::Char('C'), "/config");
        menu.add_item("About", KeyCode::Char('A'), "/about");

        match menu.select("Secrets") {
            Ok(_) => {}
            Err(err) => {
                log_error(format!("cannot select Secrets menu: {}", err));
            }
        }
        menu
    }
    pub fn index_of(&self, item: &str) -> Result<usize, Error> {
        match self.labels.iter().position(|i| i.clone().eq(item)) {
            Some(pos) => Ok(pos),
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }

    pub fn select_by_location(&mut self, location: String) {
        match self.router.recognize(&location) {
            Ok(item) => {
                let label = item.handler().label.clone();
                let index = match self.index_of(&label) {
                    Ok(index) => index,
                    Err(err) => {
                        log_error(format!("cannot select by location {:?}: {}", location, err));
                        0
                    }
                };
                self.selected = Some(index);
            }
            Err(error) => {
                log_error(format!("failed to select menu by location: {}", error));
            }
        }
    }
    pub fn selected_index(&self) -> usize {
        match self.selected {
            Some(selected) => selected,
            None => 0,
        }
    }
    pub fn current(&self) -> Option<MenuItem> {
        let label = self.current_label();
        match self.items.get(&label) {
            Some(item) => Some(item.clone()),
            None => None,
        }
    }
    pub fn current_label(&self) -> String {
        self.labels[self.selected_index()].clone()
    }

    pub fn set_index(&mut self, index: usize) {
        self.selected = Some(index);
    }

    pub fn select(&mut self, item: &str) -> Result<(), Error> {
        match self.index_of(item.clone()) {
            Ok(index) => {
                self.set_index(index);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    pub fn next(&mut self) {
        let count = self.labels.len();
        // log_error(format!(
        //     "Menu.next (before) [selected={:?}] [count={:?}] ",
        //     self.selected, count
        // ));
        match self.selected {
            Some(selected) => {
                self.selected = Some((selected + 1) % count);
            }
            None => {
                if self.labels.len() > 0 {
                    self.selected = Some(0);
                }
            }
        }
        log_error(format!(
            "Menu.next (after) [selected={:?}] [count={:?}] ",
            self.selected, count
        ));
    }
    pub fn previous(&mut self) {
        let count = self.labels.len();
        // log_error(format!(
        //     "Menu.previous (before) [selected={:?}] [count={:?}] ",
        //     self.selected, count
        // ));
        match self.selected {
            Some(selected) => {
                if selected > 0 {
                    self.selected = Some(selected - 1);
                } else {
                    let last = count - 1;
                    self.selected = Some(last);
                }
            }
            None => {
                let count = self.labels.len();
                if count > 0 {
                    self.selected = Some(count - 1);
                }
            }
        }
        log_error(format!(
            "Menu.previous (after) [selected={:?}] [count={:?}] ",
            self.selected, count
        ));
    }
    pub fn add_item(&mut self, title: &str, code: KeyCode, route_path: &str) {
        let label = String::from(title);
        let item = MenuItem::new(label.clone(), code, String::from(route_path));
        self.labels.push(label.clone());
        self.items.insert(label, item.clone());
        self.router.add(route_path, item);
        if self.selected == None {
            self.selected = Some(0)
        }
    }
    pub fn remove_item(&mut self, item: &str) -> Result<(), Error> {
        match self.index_of(item) {
            Ok(index) => {
                self.labels.remove(index);
                self.items.remove(item);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
    pub fn render_in_parent(
        &mut self,
        parent: &mut Frame<CrosstermBackend<io::Stdout>>,
        chunk: Rect,
    ) -> Result<(), Error> {
        let menu = self
            .labels
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Spans::from(vec![
                    Span::styled(
                        first,
                        ui::default_style()
                            .fg(ui::color_default())
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, ui::default_style().fg(ui::color_blurred())),
                ])
            })
            .collect();
        let tabs = Tabs::new(menu)
            .select(self.selected_index())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(ui::default_style().fg(ui::color_blurred()))
            .highlight_style(ui::default_style().fg(ui::color_light()))
            .divider(Span::raw("|"));

        parent.render_widget(tabs, chunk);

        Ok(())
    }
}
impl Component for Menu {
    fn name(&self) -> &str {
        "Menu"
    }
    fn id(&self) -> String {
        self.cid.clone()
    }

    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        match event.code {
            KeyCode::Right => {
                self.next();
                return match self.current() {
                    Some(selected) => {
                        context.borrow_mut().goto(&selected.route_path);
                        return Ok(Refresh);
                    }
                    None => Ok(Refresh),
                };
            }
            KeyCode::Left => {
                self.previous();
                return match self.current() {
                    Some(selected) => {
                        context.borrow_mut().goto(&selected.route_path);
                        return Ok(Refresh);
                    }
                    None => Ok(Refresh),
                };
            }
            code => {
                if event.modifiers == KeyModifiers::CONTROL && code == KeyCode::Char('q') {
                    return Ok(Quit);
                }
                if code == KeyCode::Esc {
                    context.borrow_mut().goto("/");
                    return Ok(Refresh);
                }

                for (label, item) in self.items.iter() {
                    let label = label.clone();
                    if item.code == code {
                        match self.select(&label) {
                            Ok(_) => return Ok(Refresh),
                            Err(error) => {
                                log_error(format!("Menu.process_keyboard(): {}", error));
                                return Ok(Quit);
                            }
                        };
                    }
                }
            }
        }
        Ok(Propagate)
    }
}
