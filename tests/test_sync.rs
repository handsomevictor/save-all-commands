use sac::store::{Command, Store};
use sac::sync::{diff_stores, parse_remote};

fn make_command(id: u32, cmd: &str) -> Command {
    Command {
        id,
        folder: "f1".to_string(),
        cmd: cmd.to_string(),
        desc: format!("desc {}", id),
        comment: String::new(),
        tags: vec![],
        last_used: String::new(),
    }
}

#[test]
fn test_parse_remote_valid() {
    let toml_content = r#"
[[folders]]
id = "f1"
parent = ""
name = "Folder 1"

[[commands]]
id = 1
folder = "f1"
cmd = "ls -la"
desc = "List files"
"#;
    let store = parse_remote(toml_content).expect("Should parse valid TOML");
    assert_eq!(store.folders.len(), 1);
    assert_eq!(store.commands.len(), 1);
    assert_eq!(store.commands[0].cmd, "ls -la");
}

#[test]
fn test_parse_remote_invalid() {
    let invalid = "this is not = = valid toml !!!";
    let result = parse_remote(invalid);
    assert!(result.is_err(), "Expected error for invalid TOML");
}

#[test]
fn test_diff_stores_new_commands() {
    let local = Store::default();
    let mut remote = Store::default();
    remote.commands.push(make_command(1, "ls -la"));
    remote.commands.push(make_command(2, "pwd"));

    let (new_ids, modified_ids, _) = diff_stores(&local, &remote);
    assert_eq!(new_ids.len(), 2);
    assert!(new_ids.contains(&1));
    assert!(new_ids.contains(&2));
    assert!(modified_ids.is_empty());
}

#[test]
fn test_diff_stores_modified_commands() {
    let mut local = Store::default();
    local.commands.push(make_command(1, "ls -la"));

    let mut remote = Store::default();
    remote.commands.push(make_command(1, "ls -lah")); // modified

    let (new_ids, modified_ids, conflict_ids) = diff_stores(&local, &remote);
    assert!(new_ids.is_empty());
    assert_eq!(modified_ids.len(), 1);
    assert!(modified_ids.contains(&1));
    assert_eq!(conflict_ids.len(), 1);
}

#[test]
fn test_diff_stores_identical() {
    let mut local = Store::default();
    local.commands.push(make_command(1, "ls -la"));

    let mut remote = Store::default();
    remote.commands.push(make_command(1, "ls -la")); // same

    let (new_ids, modified_ids, conflict_ids) = diff_stores(&local, &remote);
    assert!(new_ids.is_empty());
    assert!(modified_ids.is_empty());
    assert!(conflict_ids.is_empty());
}
