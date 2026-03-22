use sac::store::{Command, Folder, Store};

fn make_folder(id: &str, parent: &str, name: &str) -> Folder {
    Folder {
        id: id.to_string(),
        parent: parent.to_string(),
        name: name.to_string(),
    }
}

fn make_command(id: u32, folder: &str) -> Command {
    Command {
        id,
        folder: folder.to_string(),
        cmd: format!("cmd_{}", id),
        desc: format!("desc_{}", id),
        comment: String::new(),
        tags: vec![],
        last_used: String::new(),
    }
}

// ── combined item limit ────────────────────────────────────────────────────

#[test]
fn test_validate_empty_store() {
    let store = Store::default();
    assert!(store.validate().is_ok());
}

#[test]
fn test_validate_combined_limit_ok() {
    // 4 sub-folders + 6 commands = 10 total → OK
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    for i in 0..4 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }
    for i in 1..=6 {
        store.commands.push(make_command(i, "root"));
    }
    assert!(store.validate().is_ok(), "4 folders + 6 commands = 10, should pass");
}

#[test]
fn test_validate_combined_limit_exceeded() {
    // 5 sub-folders + 6 commands = 11 total → fails
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    for i in 0..5 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }
    for i in 1..=6 {
        store.commands.push(make_command(i, "root"));
    }
    let result = store.validate();
    assert!(result.is_err(), "5 folders + 6 commands = 11, should fail");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("11") || msg.contains("limit"),
        "Error should mention count or limit: {}",
        msg
    );
}

#[test]
fn test_validate_command_limit_ok() {
    // 0 sub-folders + 10 commands = 10 total → OK
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));
    for i in 1..=10 {
        store.commands.push(make_command(i, "f1"));
    }
    assert!(store.validate().is_ok(), "10 commands with no sub-folders should pass");
}

#[test]
fn test_validate_command_limit_exceeded() {
    // 0 sub-folders + 11 commands = 11 total → fails
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));
    for i in 1..=11 {
        store.commands.push(make_command(i, "f1"));
    }
    let result = store.validate();
    assert!(result.is_err(), "11 commands should fail validation");
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("11") || msg.contains("limit"),
        "Error should mention count or limit: {}",
        msg
    );
}

#[test]
fn test_validate_subfolder_limit_ok() {
    // 10 sub-folders + 0 commands = 10 total → OK
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    for i in 0..10 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }
    assert!(store.validate().is_ok(), "10 sub-folders with no commands should pass");
}

#[test]
fn test_validate_subfolder_limit_exceeded() {
    // 11 sub-folders + 0 commands = 11 total → fails
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));
    for i in 0..11 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }
    assert!(store.validate().is_err(), "11 sub-folders should fail validation");
}

#[test]
fn test_validate_multiple_folders_independent() {
    // f1: 10 items, f2: 10 items — each folder checked independently
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));
    store.folders.push(make_folder("f2", "", "Folder 2"));
    for i in 1..=10 {
        store.commands.push(make_command(i, "f1"));
    }
    for i in 11..=20 {
        store.commands.push(make_command(i, "f2"));
    }
    assert!(
        store.validate().is_ok(),
        "Two folders each with 10 items should both be valid"
    );
}

// ── auto_fix_ids ───────────────────────────────────────────────────────────

#[test]
fn test_auto_fix_ids_no_duplicates() {
    let mut store = Store::default();
    store.commands.push(make_command(1, "f1"));
    store.commands.push(make_command(2, "f1"));
    store.commands.push(make_command(3, "f1"));
    let changed = store.auto_fix_ids();
    assert!(!changed, "No duplicates → auto_fix_ids should return false");
    assert_eq!(store.commands[0].id, 1);
    assert_eq!(store.commands[1].id, 2);
    assert_eq!(store.commands[2].id, 3);
}

#[test]
fn test_auto_fix_ids_with_duplicates() {
    let mut store = Store::default();
    store.commands.push(make_command(1, "f1"));
    store.commands.push(make_command(1, "f1")); // duplicate
    store.commands.push(make_command(2, "f1"));
    let changed = store.auto_fix_ids();
    assert!(changed, "Duplicates found → auto_fix_ids should return true");
    // IDs must be unique after fix
    let ids: Vec<u32> = store.commands.iter().map(|c| c.id).collect();
    let unique: std::collections::HashSet<u32> = ids.iter().cloned().collect();
    assert_eq!(ids.len(), unique.len(), "All IDs must be unique after fix");
}

#[test]
fn test_auto_fix_ids_all_same() {
    // Extreme case: all commands have id = 0
    let mut store = Store::default();
    for _ in 0..5 {
        store.commands.push(make_command(0, "f1"));
    }
    let changed = store.auto_fix_ids();
    assert!(changed, "All-same IDs should be fixed");
    let ids: Vec<u32> = store.commands.iter().map(|c| c.id).collect();
    let unique: std::collections::HashSet<u32> = ids.iter().cloned().collect();
    assert_eq!(ids.len(), unique.len(), "All IDs must be unique after fix");
}
