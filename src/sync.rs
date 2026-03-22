use anyhow::Result;
use std::net::TcpStream;
use std::time::Duration;

use crate::config::Config;
use crate::store::Store;

/// Try connecting to 1.1.1.1:80 with a 2-second timeout to check network.
pub fn check_network() -> bool {
    TcpStream::connect_timeout(
        &"1.1.1.1:80".parse().unwrap(),
        Duration::from_secs(2),
    )
    .is_ok()
}

/// HTTP GET using reqwest blocking client.
pub fn fetch_remote(url: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    let response = client.get(url).send()?;
    let text = response.text()?;
    Ok(text)
}

/// Parse TOML content as Store.
pub fn parse_remote(content: &str) -> Result<Store> {
    let store: Store = toml::from_str(content)?;
    Ok(store)
}

/// Returns (new_ids, modified_ids, conflict_ids).
/// - new: ids in remote but not in local
/// - modified: ids in both but cmd differs
/// - conflicts: same as modified for now
pub fn diff_stores(local: &Store, remote: &Store) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let mut new_ids = Vec::new();
    let mut modified_ids = Vec::new();

    for remote_cmd in &remote.commands {
        match local.commands.iter().find(|c| c.id == remote_cmd.id) {
            None => new_ids.push(remote_cmd.id),
            Some(local_cmd) => {
                if local_cmd.cmd != remote_cmd.cmd {
                    modified_ids.push(remote_cmd.id);
                }
            }
        }
    }

    let conflict_ids = modified_ids.clone();
    (new_ids, modified_ids, conflict_ids)
}

/// Full sync flow: check mode is "remote", check network, fetch, parse, diff, show results,
/// prompt y/n, if y save. No network: return Ok(()) silently.
pub fn sync_check(config: &Config, store: &Store) -> Result<()> {
    if config.commands_source.mode != "remote" {
        return Ok(());
    }

    if config.commands_source.url.is_empty() {
        return Ok(());
    }

    if !check_network() {
        return Ok(());
    }

    let content = fetch_remote(&config.commands_source.url)?;
    let remote_store = parse_remote(&content)?;

    let (new_ids, modified_ids, _conflict_ids) = diff_stores(store, &remote_store);

    println!("Sync results:");
    println!("  New commands: {}", new_ids.len());
    println!("  Modified commands: {}", modified_ids.len());

    if new_ids.is_empty() && modified_ids.is_empty() {
        println!("Already up to date.");
        return Ok(());
    }

    eprint!("Apply changes? [y/N] ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    if input.trim().to_lowercase() == "y" {
        remote_store.save()?;
        println!("Sync complete.");
    } else {
        println!("Sync cancelled.");
    }

    Ok(())
}
