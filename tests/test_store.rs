use sac::store::{Command, Folder, Store};
use tempfile::tempdir;

fn make_command(id: u32, folder: &str, cmd: &str) -> Command {
    Command {
        id,
        folder: folder.to_string(),
        cmd: cmd.to_string(),
        desc: format!("desc {}", id),
        comment: String::new(),
        tags: vec![],
        last_used: String::new(),
    }
}

fn make_folder(id: &str, parent: &str, name: &str) -> Folder {
    Folder {
        id: id.to_string(),
        parent: parent.to_string(),
        name: name.to_string(),
    }
}

#[test]
fn test_load_store_success() {
    let dir = tempdir().expect("tempdir failed");
    let path = dir.path().join("commands.toml");

    let toml_content = r#"
[[folders]]
id = "f1"
parent = ""
name = "Root Folder"

[[commands]]
id = 1
folder = "f1"
cmd = "ls -la"
desc = "List files"
"#;
    std::fs::write(&path, toml_content).expect("write failed");

    let store = Store::load_from(&path).expect("load failed");
    assert_eq!(store.folders.len(), 1);
    assert_eq!(store.commands.len(), 1);
    assert_eq!(store.folders[0].name, "Root Folder");
    assert_eq!(store.commands[0].cmd, "ls -la");
}

#[test]
fn test_load_store_malformed_toml() {
    let dir = tempdir().expect("tempdir failed");
    let path = dir.path().join("commands.toml");
    std::fs::write(&path, "this is not valid toml = = =").expect("write failed");

    let result = Store::load_from(&path);
    assert!(result.is_err(), "Expected error for malformed TOML");
}

#[test]
fn test_load_store_missing_file_returns_empty() {
    let dir = tempdir().expect("tempdir failed");
    let path = dir.path().join("nonexistent.toml");

    let store = Store::load_from(&path).expect("Should return empty store for missing file");
    assert!(store.folders.is_empty());
    assert!(store.commands.is_empty());
}

#[test]
fn test_validate_folder_limit_ok() {
    let mut store = Store::default();
    // Add exactly 10 subfolders under "root"
    store.folders.push(make_folder("root", "", "Root"));
    for i in 0..10 {
        store
            .folders
            .push(make_folder(&format!("f{}", i), "root", &format!("Child {}", i)));
    }

    let result = store.validate();
    assert!(result.is_ok(), "Expected validation to pass: {:?}", result);
}

#[test]
fn test_validate_folder_limit_exceeded() {
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    // Add 11 subfolders under "root" - should exceed limit
    for i in 0..11 {
        store
            .folders
            .push(make_folder(&format!("f{}", i), "root", &format!("Child {}", i)));
    }

    let result = store.validate();
    assert!(result.is_err(), "Expected validation to fail");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("11") || err_msg.contains("subfolders"),
        "Error message should mention count or subfolders: {}",
        err_msg
    );
}

#[test]
fn test_children_folders() {
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    store.folders.push(make_folder("child1", "root", "Child 1"));
    store.folders.push(make_folder("child2", "root", "Child 2"));
    store.folders.push(make_folder("other", "", "Other Root"));

    let children = store.children_folders("root");
    assert_eq!(children.len(), 2);
    let names: Vec<&str> = children.iter().map(|f| f.name.as_str()).collect();
    assert!(names.contains(&"Child 1"));
    assert!(names.contains(&"Child 2"));

    let root_children = store.children_folders("");
    assert_eq!(root_children.len(), 2);
}

#[test]
fn test_folder_commands() {
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));
    store.folders.push(make_folder("f2", "", "Folder 2"));
    store.commands.push(make_command(1, "f1", "ls"));
    store.commands.push(make_command(2, "f1", "pwd"));
    store.commands.push(make_command(3, "f2", "echo hello"));

    let f1_cmds = store.folder_commands("f1");
    assert_eq!(f1_cmds.len(), 2);

    let f2_cmds = store.folder_commands("f2");
    assert_eq!(f2_cmds.len(), 1);
    assert_eq!(f2_cmds[0].cmd, "echo hello");
}

#[test]
fn test_breadcrumb() {
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    store.folders.push(make_folder("sub", "root", "Sub"));
    store.folders.push(make_folder("deep", "sub", "Deep"));

    let crumb = store.breadcrumb("deep");
    assert_eq!(crumb, vec!["Root", "Sub", "Deep"]);

    let crumb_root = store.breadcrumb("root");
    assert_eq!(crumb_root, vec!["Root"]);

    // Non-existent folder returns empty
    let crumb_missing = store.breadcrumb("nonexistent");
    assert!(crumb_missing.is_empty());
}

#[test]
fn test_next_command_id() {
    let mut store = Store::default();
    assert_eq!(store.next_command_id(), 1);

    store.commands.push(make_command(3, "f1", "cmd"));
    store.commands.push(make_command(1, "f1", "cmd2"));
    assert_eq!(store.next_command_id(), 4);
}

#[test]
fn test_save_load_roundtrip() {
    let dir = tempdir().expect("tempdir failed");
    let path = dir.path().join("commands.toml");

    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "My Folder"));
    store.commands.push(Command {
        id: 1,
        folder: "f1".to_string(),
        cmd: "git status".to_string(),
        desc: "Check git status".to_string(),
        comment: "useful".to_string(),
        tags: vec!["git".to_string()],
        last_used: "2024-01-01".to_string(),
    });

    store.save_to(&path).expect("save failed");
    let loaded = Store::load_from(&path).expect("load failed");

    assert_eq!(loaded.folders.len(), 1);
    assert_eq!(loaded.commands.len(), 1);
    assert_eq!(loaded.commands[0].cmd, "git status");
    assert_eq!(loaded.commands[0].tags, vec!["git"]);
    assert_eq!(loaded.commands[0].comment, "useful");
}
