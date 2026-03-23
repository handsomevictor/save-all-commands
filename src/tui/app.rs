use anyhow::Result;
use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::search::{SearchResult, Searcher};
use crate::store::{Command, Folder, Store};

pub enum Mode {
    Browse,
    Search,
}

pub enum SearchMode {
    Fuzzy,
    Exact,
}

#[derive(Clone)]
pub enum BrowseItem {
    Folder(Folder),
    Command(Command),
}

pub struct App {
    pub mode: Mode,
    pub search_mode: SearchMode,
    pub current_folder: String,
    pub items: Vec<BrowseItem>,
    pub selected_index: usize,
    pub breadcrumb: Vec<String>,
    pub search_query: String,
    pub search_results: Vec<SearchResult>,
    pub search_selected: usize,
    pub output: Option<String>,
    pub should_quit: bool,
    pub store: Store,
    pub searcher: Searcher,
}

impl App {
    pub fn new(store: Store) -> Self {
        let mut app = Self {
            mode: Mode::Browse,
            search_mode: SearchMode::Fuzzy,
            current_folder: String::new(),
            items: Vec::new(),
            selected_index: 0,
            breadcrumb: Vec::new(),
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            output: None,
            should_quit: false,
            store,
            searcher: Searcher::new(),
        };
        app.load_items();
        app
    }

    pub fn handle_key(&mut self, key_event: KeyEvent) -> Result<()> {
        match self.mode {
            Mode::Browse => self.handle_key_browse(key_event),
            Mode::Search => self.handle_key_search(key_event),
        }
    }

    fn handle_key_browse(&mut self, key_event: KeyEvent) -> Result<()> {
        // Ctrl+C always quits without output
        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && key_event.code == KeyCode::Char('c')
        {
            self.should_quit = true;
            return Ok(());
        }
        match key_event.code {
            KeyCode::Char('1') => self.select_digit(0)?,
            KeyCode::Char('2') => self.select_digit(1)?,
            KeyCode::Char('3') => self.select_digit(2)?,
            KeyCode::Char('4') => self.select_digit(3)?,
            KeyCode::Char('5') => self.select_digit(4)?,
            KeyCode::Char('6') => self.select_digit(5)?,
            KeyCode::Char('7') => self.select_digit(6)?,
            KeyCode::Char('8') => self.select_digit(7)?,
            KeyCode::Char('9') => self.select_digit(8)?,
            KeyCode::Char('0') => self.select_digit(9)?,
            KeyCode::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                let max = if self.items.is_empty() { 0 } else { self.items.len() - 1 };
                if self.selected_index < max {
                    self.selected_index += 1;
                }
            }
            KeyCode::Enter => {
                self.confirm_selected()?;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.go_back()?;
            }
            KeyCode::Backspace => {
                // no-op in browse when query empty
            }
            KeyCode::Char(c) => {
                // Any printable char: append to search_query, switch to Search mode
                self.search_query.push(c);
                self.mode = Mode::Search;
                self.search_selected = 0;
                self.refresh_search()?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_search(&mut self, key_event: KeyEvent) -> Result<()> {
        // Ctrl+C always quits without output
        if key_event.modifiers.contains(KeyModifiers::CONTROL)
            && key_event.code == KeyCode::Char('c')
        {
            self.should_quit = true;
            return Ok(());
        }
        match key_event.code {
            KeyCode::Char('1') => self.select_search_digit(0)?,
            KeyCode::Char('2') => self.select_search_digit(1)?,
            KeyCode::Char('3') => self.select_search_digit(2)?,
            KeyCode::Char('4') => self.select_search_digit(3)?,
            KeyCode::Char('5') => self.select_search_digit(4)?,
            KeyCode::Char('6') => self.select_search_digit(5)?,
            KeyCode::Char('7') => self.select_search_digit(6)?,
            KeyCode::Char('8') => self.select_search_digit(7)?,
            KeyCode::Char('9') => self.select_search_digit(8)?,
            KeyCode::Char('0') => self.select_search_digit(9)?,
            KeyCode::Up => {
                if self.search_selected > 0 {
                    self.search_selected -= 1;
                }
            }
            KeyCode::Down => {
                let max = if self.search_results.is_empty() {
                    0
                } else {
                    self.search_results.len() - 1
                };
                if self.search_selected < max {
                    self.search_selected += 1;
                }
            }
            KeyCode::Enter => {
                if self.search_selected < self.search_results.len() {
                    let cmd = self.search_results[self.search_selected].command.clone();
                    self.confirm_command(cmd)?;
                }
            }
            KeyCode::Esc => {
                self.search_query.clear();
                self.mode = Mode::Browse;
                self.search_results.clear();
                self.search_selected = 0;
            }
            KeyCode::Backspace => {
                // Remove last char (UTF-8 safe via chars pop)
                let mut chars: Vec<char> = self.search_query.chars().collect();
                if !chars.is_empty() {
                    chars.pop();
                    self.search_query = chars.into_iter().collect();
                }
                self.refresh_search()?;
                if self.search_query.is_empty() {
                    self.mode = Mode::Browse;
                    self.search_results.clear();
                    self.search_selected = 0;
                }
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.refresh_search()?;
                self.update_search_mode();
            }
            _ => {}
        }
        Ok(())
    }

    fn select_digit(&mut self, idx: usize) -> Result<()> {
        // Unified numbering: idx is a direct position in the items list.
        // Folders and commands share the same sequence (1-9, 0 = index 0-9).
        if idx >= self.items.len() {
            return Ok(());
        }
        match self.items[idx].clone() {
            BrowseItem::Folder(f) => self.enter_folder(f.id.clone())?,
            BrowseItem::Command(c) => self.confirm_command(c)?,
        }
        Ok(())
    }

    fn select_search_digit(&mut self, idx: usize) -> Result<()> {
        if idx < self.search_results.len() {
            let cmd = self.search_results[idx].command.clone();
            self.confirm_command(cmd)?;
        }
        Ok(())
    }

    fn confirm_selected(&mut self) -> Result<()> {
        if self.selected_index >= self.items.len() {
            return Ok(());
        }
        match self.items[self.selected_index].clone() {
            BrowseItem::Folder(f) => self.enter_folder(f.id.clone())?,
            BrowseItem::Command(c) => self.confirm_command(c)?,
        }
        Ok(())
    }

    pub fn enter_folder(&mut self, folder_id: String) -> Result<()> {
        let folder = self
            .store
            .folders
            .iter()
            .find(|f| f.id == folder_id)
            .ok_or_else(|| anyhow::anyhow!("Folder not found: {}", folder_id))?
            .clone();
        self.breadcrumb.push(folder.name.clone());
        self.current_folder = folder_id;
        self.selected_index = 0;
        self.load_items();
        Ok(())
    }

    pub fn go_back(&mut self) -> Result<()> {
        if self.breadcrumb.is_empty() {
            self.should_quit = true;
            return Ok(());
        }
        self.breadcrumb.pop();
        if self.breadcrumb.is_empty() {
            self.current_folder = String::new();
        } else {
            // Find the folder whose name matches the last breadcrumb entry
            // We need to reconstruct the folder path. Use the parent of current folder.
            let parent_id = self
                .store
                .folders
                .iter()
                .find(|f| f.id == self.current_folder)
                .map(|f| f.parent.clone())
                .unwrap_or_default();
            self.current_folder = parent_id;
        }
        self.selected_index = 0;
        self.load_items();
        Ok(())
    }

    pub fn load_items(&mut self) {
        self.items.clear();
        let folders = self.store.children_folders(&self.current_folder);
        for f in folders {
            self.items.push(BrowseItem::Folder(f.clone()));
        }
        let commands = self.store.folder_commands(&self.current_folder);
        for c in commands {
            self.items.push(BrowseItem::Command(c.clone()));
        }
        // Clamp selected_index
        if !self.items.is_empty() && self.selected_index >= self.items.len() {
            self.selected_index = self.items.len() - 1;
        }
    }

    pub fn confirm_command(&mut self, cmd: Command) -> Result<()> {
        self.output = Some(cmd.cmd.clone());
        // Update last_used
        let now = Local::now().to_rfc3339();
        if let Some(stored_cmd) = self.store.commands.iter_mut().find(|c| c.id == cmd.id) {
            stored_cmd.last_used = now;
        }
        let _ = self.store.save();
        self.should_quit = true;
        Ok(())
    }

    pub fn refresh_search(&mut self) -> Result<()> {
        self.update_search_mode();
        let effective_query = self.effective_query();
        self.search_results = match self.search_mode {
            SearchMode::Fuzzy => self.searcher.fuzzy_search(&effective_query, &self.store),
            SearchMode::Exact => self.searcher.exact_search(&effective_query, &self.store),
        };
        // Clamp search_selected
        if !self.search_results.is_empty() && self.search_selected >= self.search_results.len() {
            self.search_selected = self.search_results.len() - 1;
        }
        Ok(())
    }

    pub fn update_search_mode(&mut self) {
        if self.search_query.starts_with("//") {
            self.search_mode = SearchMode::Exact;
        } else {
            self.search_mode = SearchMode::Fuzzy;
        }
    }

    /// Returns the effective query string for searching (strips "//" prefix for exact mode).
    pub fn effective_query(&self) -> String {
        match self.search_mode {
            SearchMode::Exact => self.search_query.trim_start_matches("//").to_string(),
            SearchMode::Fuzzy => self.search_query.clone(),
        }
    }
}

