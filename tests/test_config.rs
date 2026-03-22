use sac::config::Config;
use tempfile::tempdir;

// Helper to create a Config loaded from a temp directory by temporarily
// overriding the path logic. Since Config::load() uses dirs::home_dir(),
// we test the internal save/load using the TOML serialization directly.

fn config_from_toml(toml_str: &str) -> anyhow::Result<Config> {
    let config: Config = toml::from_str(toml_str)?;
    Ok(config)
}

fn config_to_toml(config: &Config) -> anyhow::Result<String> {
    Ok(toml::to_string_pretty(config)?)
}

#[test]
fn test_config_default_values() {
    let config = Config::default();
    assert!(config.general.auto_check_remote);
    assert_eq!(config.general.last_check, "");
    assert_eq!(config.commands_source.mode, "local");
    assert_eq!(config.commands_source.path, "~/.sac/commands.toml");
    assert_eq!(config.commands_source.url, "");
    assert_eq!(config.shell.shell_type, "zsh");
}

#[test]
fn test_config_default_creation() {
    // Test that a missing config file results in default values being returned
    // We simulate this by checking that default config has the right values
    let config = Config::default();
    assert!(config.general.auto_check_remote);
    assert_eq!(config.commands_source.mode, "local");
    assert_eq!(config.shell.shell_type, "zsh");
}

#[test]
fn test_config_set_valid_key() {
    let mut config = Config::default();

    config.set("general.auto_check_remote", "false").expect("set failed");
    assert!(!config.general.auto_check_remote);

    config.set("general.last_check", "2024-01-15").expect("set failed");
    assert_eq!(config.general.last_check, "2024-01-15");

    config.set("commands_source.mode", "remote").expect("set failed");
    assert_eq!(config.commands_source.mode, "remote");

    config
        .set("commands_source.path", "/custom/path.toml")
        .expect("set failed");
    assert_eq!(config.commands_source.path, "/custom/path.toml");

    config
        .set("commands_source.url", "https://example.com/cmds.toml")
        .expect("set failed");
    assert_eq!(config.commands_source.url, "https://example.com/cmds.toml");

    config.set("shell.type", "bash").expect("set failed");
    assert_eq!(config.shell.shell_type, "bash");
}

#[test]
fn test_config_set_invalid_key() {
    let mut config = Config::default();
    let result = config.set("nonexistent.key", "value");
    assert!(result.is_err(), "Expected error for unknown key");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("nonexistent.key") || err_msg.contains("Unknown"),
        "Error should mention the bad key: {}",
        err_msg
    );
}

#[test]
fn test_config_set_invalid_bool() {
    let mut config = Config::default();
    let result = config.set("general.auto_check_remote", "not_a_bool");
    assert!(result.is_err(), "Expected error for invalid boolean value");
}

#[test]
fn test_config_save_load_roundtrip() {
    let dir = tempdir().expect("tempdir failed");
    let path = dir.path().join("config.toml");

    let mut config = Config::default();
    config.general.auto_check_remote = false;
    config.general.last_check = "2024-06-01".to_string();
    config.commands_source.mode = "remote".to_string();
    config.commands_source.url = "https://example.com/cmds.toml".to_string();
    config.shell.shell_type = "bash".to_string();

    // Serialize and write manually (since save() uses home dir)
    let toml_str = config_to_toml(&config).expect("serialize failed");
    std::fs::write(&path, &toml_str).expect("write failed");

    // Load back
    let toml_contents = std::fs::read_to_string(&path).expect("read failed");
    let loaded = config_from_toml(&toml_contents).expect("parse failed");

    assert!(!loaded.general.auto_check_remote);
    assert_eq!(loaded.general.last_check, "2024-06-01");
    assert_eq!(loaded.commands_source.mode, "remote");
    assert_eq!(loaded.commands_source.url, "https://example.com/cmds.toml");
    assert_eq!(loaded.shell.shell_type, "bash");
}
