use super::AES256Secret;

use tui::widgets::ListState;

pub struct StatefulList {
    pub state: ListState,
    pub items: Vec<AES256Secret>,
}

impl StatefulList {
    pub fn with_items(items: Vec<AES256Secret>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }
    pub fn empty() -> StatefulList {
        StatefulList::with_items(Vec::new())
    }

    pub fn update(&mut self, items: Vec<AES256Secret>) {
        self.items = items;
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn current(&mut self) -> Option<AES256Secret> {
        match self.state.selected() {
            Some(index) => {
                if self.items.len() < index + 1 {
                    return None;
                }
                Some(self.items[index].clone())
            }
            None => None,
        }
    }
    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}
