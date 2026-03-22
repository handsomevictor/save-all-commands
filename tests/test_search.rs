use sac::search::Searcher;
use sac::store::{Command, Folder, Store};

fn make_store() -> Store {
    Store {
        folders: vec![Folder {
            id: "devops".into(),
            parent: "".into(),
            name: "DevOps".into(),
        }],
        commands: vec![
            Command {
                id: 1,
                folder: "devops".into(),
                cmd: "kubectl get pods".into(),
                desc: "列出所有 pods".into(),
                comment: "".into(),
                tags: vec!["k8s".into()],
                last_used: "".into(),
            },
            Command {
                id: 2,
                folder: "devops".into(),
                cmd: "docker ps -a".into(),
                desc: "列出所有容器".into(),
                comment: "".into(),
                tags: vec![],
                last_used: "".into(),
            },
        ],
    }
}

#[test]
fn test_fuzzy_search_returns_results() {
    let store = make_store();
    let mut searcher = Searcher::new();
    let results = searcher.fuzzy_search("kubectl", &store);
    assert!(!results.is_empty(), "Expected at least one result for 'kubectl'");
    assert_eq!(
        results[0].command.cmd, "kubectl get pods",
        "First result should be the kubectl command"
    );
}

#[test]
fn test_fuzzy_search_no_match() {
    let store = make_store();
    let mut searcher = Searcher::new();
    let results = searcher.fuzzy_search("zzznomatch999", &store);
    assert!(results.is_empty(), "Expected no results for 'zzznomatch999'");
}

#[test]
fn test_exact_search_chinese() {
    let store = make_store();
    let searcher = Searcher::new();
    let results = searcher.exact_search("列出所有", &store);
    assert_eq!(
        results.len(),
        2,
        "Expected 2 results for Chinese query '列出所有', got {}",
        results.len()
    );
}

#[test]
fn test_exact_search_case_insensitive() {
    let store = make_store();
    let searcher = Searcher::new();
    let results = searcher.exact_search("KUBECTL", &store);
    assert!(!results.is_empty(), "Expected results for 'KUBECTL' (case-insensitive)");
    assert_eq!(
        results[0].command.cmd, "kubectl get pods",
        "First result should be the kubectl command"
    );
}

#[test]
fn test_fuzzy_search_empty_query() {
    let store = make_store();
    let mut searcher = Searcher::new();
    let results = searcher.fuzzy_search("", &store);
    assert_eq!(
        results.len(),
        store.commands.len(),
        "Empty query should return all commands"
    );
    // Should be ordered by id
    assert_eq!(results[0].command.id, 1);
    assert_eq!(results[1].command.id, 2);
}

#[test]
fn test_exact_search_empty_query() {
    let store = make_store();
    let searcher = Searcher::new();
    let results = searcher.exact_search("", &store);
    assert_eq!(
        results.len(),
        store.commands.len(),
        "Empty query should return all commands"
    );
    // Should be ordered by id
    assert_eq!(results[0].command.id, 1);
    assert_eq!(results[1].command.id, 2);
}

#[test]
fn test_search_result_has_folder_path() {
    let store = make_store();
    let mut searcher = Searcher::new();
    let results = searcher.fuzzy_search("kubectl", &store);
    assert!(!results.is_empty(), "Expected results");
    let result = &results[0];
    assert_eq!(
        result.folder_path,
        vec!["DevOps"],
        "folder_path should be ['DevOps'], got {:?}",
        result.folder_path
    );
}
