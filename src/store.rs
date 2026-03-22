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

    pub fn validate(&self) -> Result<()> {
        // Check max 10 subfolders per folder
        let all_parent_ids: Vec<&str> = self.folders.iter().map(|f| f.parent.as_str()).collect();
        for folder in &self.folders {
            let child_count = all_parent_ids
                .iter()
                .filter(|&&pid| pid == folder.id.as_str())
                .count();
            if child_count > 10 {
                bail!(
                    "Folder '{}' has {} subfolders, which exceeds the limit of 10",
                    folder.name,
                    child_count
                );
            }
        }

        // Also check root-level folders (parent = "")
        let root_child_count = all_parent_ids.iter().filter(|&&pid| pid.is_empty()).count();
        if root_child_count > 10 {
            bail!(
                "Root has {} subfolders, which exceeds the limit of 10",
                root_child_count
            );
        }

        // Check max 10 commands per folder
        for folder in &self.folders {
            let cmd_count = self
                .commands
                .iter()
                .filter(|c| c.folder == folder.id)
                .count();
            if cmd_count > 10 {
                bail!(
                    "Folder '{}' has {} commands, which exceeds the limit of 10",
                    folder.name,
                    cmd_count
                );
            }
        }

        Ok(())
    }
}
