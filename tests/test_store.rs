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

#[test]
fn test_cmd_backslash_roundtrip() {
    use sac::store::{Command, Folder, Store};
    use tempfile::NamedTempFile;

    let original_cmd = "curl -X POST \"https://example.com\" \\\n  -H \"accept: application/json\" \\\n  -d '{\"key\": \"value\"}'".to_string();

    let mut store = Store::default();
    store.folders.push(Folder { id: "test".to_string(), parent: "".to_string(), name: "Test".to_string() });
    store.commands.push(Command {
        id: 1, folder: "test".to_string(),
        cmd: original_cmd.clone(),
        desc: "test".to_string(), comment: "".to_string(),
        tags: vec![], last_used: "".to_string(),
    });

    let tmp = NamedTempFile::new().unwrap();
    store.save_to(tmp.path()).unwrap();

    let saved_toml = std::fs::read_to_string(tmp.path()).unwrap();
    eprintln!("=== Saved TOML ===\n{}", saved_toml);

    let reloaded = Store::load_from(tmp.path()).unwrap();
    let reloaded_cmd = &reloaded.commands[0].cmd;

    eprintln!("=== Original ===\n{}", original_cmd);
    eprintln!("=== Reloaded ===\n{}", reloaded_cmd);

    assert_eq!(*reloaded_cmd, original_cmd, "backslashes must survive save/load round-trip");
}

#[test]
fn test_cmd_output_bytes() {
    use sac::store::Store;
    let store = Store::load().unwrap();
    let cmd = store.commands.iter()
        .find(|c| c.cmd.contains("orderbookl2"))
        .expect("command not found");
    eprintln!("=== cmd string (escaped) ===");
    eprintln!("{:?}", cmd.cmd);
    eprintln!("=== cmd string (display) ===");
    eprintln!("{}", cmd.cmd);
    // Verify backslash is present
    assert!(cmd.cmd.contains('\\'), "cmd must contain backslash");
}

#[test]
fn test_cmd_stdout_bytes() {
    use sac::store::Store;
    use std::io::Write;
    let store = Store::load().unwrap();
    let cmd = store.commands.iter()
        .find(|c| c.cmd.contains("orderbookl2"))
        .expect("command not found");
    
    // Exactly what main.rs does: print!("{}", cmd)
    let output = format!("{}", cmd.cmd);
    
    // Check backslash is present
    let backslash_count = output.chars().filter(|&c| c == '\\').count();
    eprintln!("Backslash count in output: {}", backslash_count);
    eprintln!("First 80 chars: {:?}", &output[..80.min(output.len())]);
    
    assert!(backslash_count > 0, "output must contain backslashes, got: {:?}", output);
    
    // Write to a tmpfile and read back (simulates shell pipeline)
    let tmp = tempfile::NamedTempFile::new().unwrap();
    write!(tmp.as_file(), "{}", cmd.cmd).unwrap();
    
    let tmpfile_content = std::fs::read_to_string(tmp.path()).unwrap();
    let tmpfile_backslashes = tmpfile_content.chars().filter(|&c| c == '\\').count();
    eprintln!("Backslash count in tmpfile: {}", tmpfile_backslashes);
    assert_eq!(backslash_count, tmpfile_backslashes, "tmpfile must have same backslash count");
}

#[test]
fn test_save_produces_correct_toml() {
    use sac::store::Store;
    use tempfile::NamedTempFile;

    let store = Store::load().unwrap();
    let tmp = NamedTempFile::new().unwrap();
    store.save_to(tmp.path()).unwrap();

    let saved = std::fs::read_to_string(tmp.path()).unwrap();
    
    // Find the cmd section for orderbookl2
    let start = saved.find("orderbookl2").expect("command not found");
    // Show 200 chars around it
    let begin = start.saturating_sub(50);
    let end = (start + 300).min(saved.len());
    eprintln!("=== Saved TOML around orderbookl2 ===\n{}", &saved[begin..end]);
    
    // Re-load and check backslashes
    let reloaded = Store::load_from(tmp.path()).unwrap();
    let cmd = reloaded.commands.iter().find(|c| c.cmd.contains("orderbookl2")).unwrap();
    let bs = cmd.cmd.chars().filter(|&c| c == '\\').count();
    eprintln!("Backslash count after reload: {}", bs);
    assert!(bs > 0, "backslashes must survive save");
}
