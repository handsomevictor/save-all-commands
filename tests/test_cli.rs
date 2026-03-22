use clap::Parser;
use sac::cli::{Cli, Commands, ConfigSubcommand, WhereTarget};

#[test]
fn test_no_subcommand_parses() {
    let cli = Cli::try_parse_from(["sac"]).expect("should parse with no subcommand");
    assert!(cli.command.is_none());
}

#[test]
fn test_add_subcommand() {
    let cli = Cli::try_parse_from(["sac", "add"]).expect("should parse add");
    match cli.command {
        Some(Commands::Add { folder }) => assert!(folder.is_none()),
        _ => panic!("Expected Add command"),
    }
}

#[test]
fn test_add_with_folder() {
    let cli = Cli::try_parse_from(["sac", "add", "--folder", "foo"]).expect("should parse add --folder foo");
    match cli.command {
        Some(Commands::Add { folder }) => assert_eq!(folder, Some("foo".to_string())),
        _ => panic!("Expected Add command with folder"),
    }
}

#[test]
fn test_sync_force() {
    let cli = Cli::try_parse_from(["sac", "sync", "--force"]).expect("should parse sync --force");
    match cli.command {
        Some(Commands::Sync { force }) => assert!(force),
        _ => panic!("Expected Sync command"),
    }
}

#[test]
fn test_config_set() {
    let cli = Cli::try_parse_from(["sac", "config", "set", "key", "val"])
        .expect("should parse config set");
    match cli.command {
        Some(Commands::Config(args)) => match args.subcommand {
            Some(ConfigSubcommand::Set { key, value }) => {
                assert_eq!(key, "key");
                assert_eq!(value, "val");
            }
            _ => panic!("Expected Config Set subcommand"),
        },
        _ => panic!("Expected Config command"),
    }
}

#[test]
fn test_where_config() {
    let cli = Cli::try_parse_from(["sac", "where", "config"]).expect("should parse where config");
    match cli.command {
        Some(Commands::Where { target }) => {
            assert!(matches!(target, WhereTarget::Config));
        }
        _ => panic!("Expected Where command"),
    }
}

#[test]
fn test_where_commands_error_case() {
    let result = Cli::try_parse_from(["sac", "where", "invalid-target"]);
    assert!(result.is_err(), "Expected parse error for invalid where target");
}
