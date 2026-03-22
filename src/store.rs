use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Folder {
    pub id: String,
    pub parent: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Command {
    pub id: u32,
    pub folder: String,
    pub cmd: String,
    pub desc: String,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub last_used: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Store {
    #[serde(default)]
    pub folders: Vec<Folder>,
    #[serde(default)]
    pub commands: Vec<Command>,
}

fn default_store_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".sac").join("commands.toml"))
}

impl Store {
    pub fn load() -> Result<Self> {
        let path = default_store_path()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read store file: {}", path.display()))?;
        let store: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse store file: {}", path.display()))?;
        Ok(store)
    }

    pub fn save(&self) -> Result<()> {
        let path = default_store_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize store")?;
        std::fs::write(path, contents)
            .with_context(|| format!("Failed to write store file: {}", path.display()))?;
        Ok(())
    }

    pub fn children_folders(&self, parent_id: &str) -> Vec<&Folder> {
        self.folders
            .iter()
            .filter(|f| f.parent == parent_id)
            .collect()
    }

    pub fn folder_commands(&self, folder_id: &str) -> Vec<&Command> {
        self.commands
            .iter()
            .filter(|c| c.folder == folder_id)
            .collect()
    }

    /// Returns the names of folders from root down to (and including) the given folder_id.
    pub fn breadcrumb(&self, folder_id: &str) -> Vec<String> {
        let mut crumbs = Vec::new();
        let mut current_id = folder_id.to_string();

        loop {
            match self.folders.iter().find(|f| f.id == current_id) {
                None => break,
                Some(folder) => {
                    crumbs.push(folder.name.clone());
                    if folder.parent.is_empty() || folder.parent == current_id {
                        break;
                    }
                    current_id = folder.parent.clone();
                }
            }
        }

        crumbs.reverse();
        crumbs
    }

    pub fn next_command_id(&self) -> u32 {
        self.commands.iter().map(|c| c.id).max().unwrap_or(0) + 1
    }

    /// Check structural constraints.
    /// Each folder (and the root level) may contain at most 10 items total —
    /// sub-folders and commands combined — because the TUI only has keys 1-9 and 0.
    pub fn validate(&self) -> Result<()> {
        // Check root level
        let root_folders = self.folders.iter().filter(|f| f.parent.is_empty()).count();
        let root_commands = self.commands.iter().filter(|c| c.folder.is_empty()).count();
        let root_total = root_folders + root_commands;
        if root_total > 10 {
            bail!(
                "Root has {} items (sub-folders + commands combined), limit is 10",
                root_total
            );
        }

        // Check each named folder
        for folder in &self.folders {
            let child_folders = self
                .folders
                .iter()
                .filter(|f| f.parent == folder.id)
                .count();
            let child_commands = self
                .commands
                .iter()
                .filter(|c| c.folder == folder.id)
                .count();
            let total = child_folders + child_commands;
            if total > 10 {
                bail!(
                    "Folder '{}' has {} items (sub-folders + commands combined), limit is 10",
                    folder.name,
                    total
                );
            }
        }

        Ok(())
    }

    /// Detect duplicate command IDs and reassign them sequentially.
    /// Returns true if any IDs were changed.
    pub fn auto_fix_ids(&mut self) -> bool {
        use std::collections::HashSet;
        let mut seen: HashSet<u32> = HashSet::new();
        let has_duplicates = self.commands.iter().any(|c| !seen.insert(c.id));
        if !has_duplicates {
            return false;
        }
        // Sort by current ID to preserve relative order, then reassign 1, 2, 3, ...
        self.commands.sort_by_key(|c| c.id);
        for (i, cmd) in self.commands.iter_mut().enumerate() {
            cmd.id = (i + 1) as u32;
        }
        true
    }
}
