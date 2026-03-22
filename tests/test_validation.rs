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

#[test]
fn test_validate_command_limit_ok() {
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));

    // Add exactly 10 commands - should be OK
    for i in 1..=10 {
        store.commands.push(make_command(i, "f1"));
    }

    let result = store.validate();
    assert!(result.is_ok(), "10 commands should be valid: {:?}", result);
}

#[test]
fn test_validate_command_limit_exceeded() {
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));

    // Add 11 commands - should exceed limit
    for i in 1..=11 {
        store.commands.push(make_command(i, "f1"));
    }

    let result = store.validate();
    assert!(result.is_err(), "11 commands should fail validation");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("11") || err_msg.contains("commands"),
        "Error should mention count or commands: {}",
        err_msg
    );
}

#[test]
fn test_validate_multiple_folders_independent() {
    let mut store = Store::default();
    store.folders.push(make_folder("f1", "", "Folder 1"));
    store.folders.push(make_folder("f2", "", "Folder 2"));

    // 10 commands in f1 (OK) and 10 in f2 (OK) - they should not interfere
    for i in 1..=10 {
        store.commands.push(make_command(i, "f1"));
    }
    for i in 11..=20 {
        store.commands.push(make_command(i, "f2"));
    }

    let result = store.validate();
    assert!(
        result.is_ok(),
        "Each folder with 10 commands should be valid: {:?}",
        result
    );
}

#[test]
fn test_validate_subfolder_limit_ok() {
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));

    for i in 0..10 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }

    let result = store.validate();
    assert!(
        result.is_ok(),
        "10 subfolders should be valid: {:?}",
        result
    );
}

#[test]
fn test_validate_subfolder_limit_exceeded() {
    let mut store = Store::default();
    store.folders.push(make_folder("root", "", "Root"));

    for i in 0..11 {
        store
            .folders
            .push(make_folder(&format!("child{}", i), "root", &format!("Child {}", i)));
    }

    let result = store.validate();
    assert!(result.is_err(), "11 subfolders should fail validation");
}

#[test]
fn test_validate_empty_store() {
    let store = Store::default();
    let result = store.validate();
    assert!(result.is_ok(), "Empty store should be valid: {:?}", result);
}
