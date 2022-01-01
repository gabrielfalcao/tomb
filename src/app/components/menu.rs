//use super::super::logging::log_error;
use super::super::ui;
use crate::ironpunk::*;
use crossterm::event::{KeyCode, KeyEvent};
use std::{collections::BTreeMap, io};
use tui::{
    backend::CrosstermBackend,
    layout::Rect,
    style::{Color, Modifier, Style},
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
impl MenuItem {
    pub fn new(label: String, code: KeyCode, route_path: String) -> MenuItem {
        MenuItem {
            label,
            code,
            route_path,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Menu {
    pub cid: String,
    pub selected: Option<usize>,
    pub labels: Vec<String>,
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
            error: None,
        }
    }
    pub fn default(selected: &str) -> Menu {
        let mut menu = Menu::empty();
        menu.add_item("Secrets", KeyCode::Char('s'), "/").unwrap();
        menu.add_item("About", KeyCode::Char('a'), "/about")
            .unwrap();
        menu.select(selected).unwrap();
        menu
    }
    pub fn index_of(&self, item: &str) -> Result<usize, Error> {
        match self
            .labels
            .iter()
            .position(|i| i.clone() == String::from(item))
        {
            Some(pos) => Ok(pos),
            None => Err(Error::with_message(format!("invalid menu item: {}", item))),
        }
    }

    pub fn select_by_location(&mut self, location: String) {
        // log_error(format!("select_by_location({:?})", location));
        for (_, item) in &self.items {
            if item.route_path.eq(&location) {
                let index = match self.index_of(&item.label) {
                    Ok(index) => index,
                    Err(_) => 0,
                };
                self.selected = Some(index);
                return;
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
        match self.current_label() {
            Some(label) => match self.items.get(&label) {
                Some(item) => Some(item.clone()),
                None => None,
            },
            None => None,
        }
    }
    pub fn current_label(&self) -> Option<String> {
        match self.selected {
            Some(selected) => Some(self.labels[selected].clone()),
            None => None,
        }
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
        match self.selected {
            Some(selected) => {
                if selected < self.labels.len() - 1 {
                    self.selected = Some(selected + 1);
                }
            }
            None => {}
        }
    }
    pub fn previous(&mut self) {
        match self.selected {
            Some(selected) => {
                if selected > 0 {
                    self.selected = Some(selected - 1);
                }
            }
            None => {}
        }
    }
    pub fn add_item(&mut self, title: &str, code: KeyCode, route_path: &str) -> Result<(), Error> {
        let label = String::from(title);
        let item = MenuItem::new(label.clone(), code, String::from(route_path));
        self.labels.push(label.clone());
        self.items.insert(label, item);
        if self.selected == None {
            self.selected = Some(0)
        }
        Ok(())
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
                        Style::default()
                            .fg(ui::color_default())
                            .add_modifier(Modifier::UNDERLINED),
                    ),
                    Span::styled(rest, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect();
        let tabs = Tabs::new(menu)
            .select(self.selected_index())
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(Style::default().fg(ui::color_light()))
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

    #[allow(unused_variables)]
    fn process_keyboard(
        &mut self,
        event: KeyEvent,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        context: SharedContext,
        router: SharedRouter,
    ) -> Result<LoopEvent, Error> {
        let code = event.code;

        match code {
            KeyCode::Right => {
                self.next();
                match self.current() {
                    Some(selected) => {
                        context.borrow_mut().goto(&selected.route_path);
                        return Ok(Refresh);
                    }
                    None => {}
                }
            }
            KeyCode::Left => {
                self.previous();
                match self.current() {
                    Some(selected) => {
                        context.borrow_mut().goto(&selected.route_path);
                        return Ok(Refresh);
                    }
                    None => {}
                }
            }
            code => {
                for (label, item) in &self.items {
                    let label = label.clone();
                    if item.code == code {
                        match self.select(&label) {
                            Ok(_) => return Ok(Propagate),
                            Err(error) => return Ok(Quit),
                        };
                    }
                }
            }
        }
        Ok(Refresh)
    }
}
