use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub auto_check_remote: bool,
    pub last_check: String,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            auto_check_remote: true,
            last_check: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsSourceConfig {
    pub mode: String,
    pub path: String,
    pub url: String,
}

impl Default for CommandsSourceConfig {
    fn default() -> Self {
        Self {
            mode: "local".to_string(),
            path: "~/.sac/commands.toml".to_string(),
            url: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    #[serde(rename = "type")]
    pub shell_type: String,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_type: "zsh".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub commands_source: CommandsSourceConfig,
    #[serde(default)]
    pub shell: ShellConfig,
}

fn config_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".sac").join("config.toml"))
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;

        if !path.exists() {
            let config = Config::default();
            config.save()?;
            return Ok(config);
        }

        let contents = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Config = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }
        let contents = toml::to_string_pretty(self).context("Failed to serialize config")?;
        std::fs::write(&path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;
        Ok(())
    }

    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "general.auto_check_remote" => {
                let parsed: bool = value
                    .parse()
                    .with_context(|| format!("Invalid boolean value: '{}'", value))?;
                self.general.auto_check_remote = parsed;
            }
            "general.last_check" => {
                self.general.last_check = value.to_string();
            }
            "commands_source.mode" => {
                self.commands_source.mode = value.to_string();
            }
            "commands_source.path" => {
                self.commands_source.path = value.to_string();
            }
            "commands_source.url" => {
                self.commands_source.url = value.to_string();
            }
            "shell.type" => {
                self.shell.shell_type = value.to_string();
            }
            _ => {
                bail!("Unknown config key: '{}'", key);
            }
        }
        Ok(())
    }
}
