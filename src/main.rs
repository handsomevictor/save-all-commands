use clap::Parser;

use sac::cli::{Cli, Commands, ConfigSubcommand, WhereTarget};
use sac::config::Config;
use sac::store::{Command, Folder, Store};
use sac::tui;
use sac::{shell, sync};

fn prompt(label: &str) -> std::io::Result<String> {
    eprint!("{}", label);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn handle_add(folder: Option<String>) -> anyhow::Result<()> {
    let mut store = Store::load()?;

    let folder_id = match folder {
        Some(f) => f,
        None => {
            if store.folders.is_empty() {
                eprintln!("No folders exist. Use `sac new-folder <name>` to create one first.");
                return Ok(());
            }
            eprintln!("Available folders:");
            for f in &store.folders {
                eprintln!("  {} ({})", f.name, f.id);
            }
            prompt("Folder id: ")?
        }
    };

    if folder_id.is_empty() {
        anyhow::bail!("Folder id cannot be empty");
    }

    let cmd = prompt("Command: ")?;
    if cmd.is_empty() {
        anyhow::bail!("Command cannot be empty");
    }

    let desc = prompt("Description: ")?;
    let comment = prompt("Comment (optional): ")?;
    let tags_raw = prompt("Tags (comma-separated, optional): ")?;
    let tags: Vec<String> = if tags_raw.is_empty() {
        vec![]
    } else {
        tags_raw
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty())
            .collect()
    };

    let id = store.next_command_id();
    let command = Command {
        id,
        folder: folder_id,
        cmd,
        desc,
        comment,
        tags,
        last_used: String::new(),
    };

    store.commands.push(command);
    store.save()?;
    println!("Command added with id {}", id);
    Ok(())
}

fn handle_new_folder(name: String, parent: Option<String>) -> anyhow::Result<()> {
    let mut store = Store::load()?;

    let id = match &parent {
        Some(p) => format!("{}.{}", p, name.to_lowercase().replace(' ', "_")),
        None => name.to_lowercase().replace(' ', "_"),
    };

    let folder = Folder {
        id: id.clone(),
        parent: parent.unwrap_or_default(),
        name,
    };

    store.folders.push(folder);
    store.validate()?;
    store.save()?;
    println!("Folder created with id '{}'", id);
    Ok(())
}

fn handle_edit(command_id: u32) -> anyhow::Result<()> {
    let mut store = Store::load()?;

    let pos = store
        .commands
        .iter()
        .position(|c| c.id == command_id)
        .ok_or_else(|| anyhow::anyhow!("Command with id {} not found", command_id))?;

    let current = store.commands[pos].clone();

    eprintln!("Editing command {} (press Enter to keep current value)", command_id);

    let cmd = {
        let v = prompt(&format!("Command [{}]: ", current.cmd))?;
        if v.is_empty() { current.cmd.clone() } else { v }
    };
    let desc = {
        let v = prompt(&format!("Description [{}]: ", current.desc))?;
        if v.is_empty() { current.desc.clone() } else { v }
    };
    let comment = {
        let v = prompt(&format!("Comment [{}]: ", current.comment))?;
        if v.is_empty() { current.comment.clone() } else { v }
    };
    let tags = {
        let current_tags = current.tags.join(", ");
        let v = prompt(&format!("Tags [{}]: ", current_tags))?;
        if v.is_empty() {
            current.tags.clone()
        } else {
            v.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        }
    };
    let folder = {
        let v = prompt(&format!("Folder [{}]: ", current.folder))?;
        if v.is_empty() { current.folder.clone() } else { v }
    };

    store.commands[pos] = Command {
        id: command_id,
        folder,
        cmd,
        desc,
        comment,
        tags,
        last_used: current.last_used,
    };

    store.save()?;
    println!("Command {} updated.", command_id);
    Ok(())
}

fn handle_delete(command_id: u32) -> anyhow::Result<()> {
    let mut store = Store::load()?;

    let pos = store
        .commands
        .iter()
        .position(|c| c.id == command_id)
        .ok_or_else(|| anyhow::anyhow!("Command with id {} not found", command_id))?;

    let cmd = &store.commands[pos];
    eprintln!("Command #{}: {} - {}", cmd.id, cmd.cmd, cmd.desc);

    let answer = prompt("Delete? [y/N] ")?;
    if answer.to_lowercase() == "y" {
        store.commands.remove(pos);
        store.save()?;
        println!("Command {} deleted.", command_id);
    } else {
        println!("Delete cancelled.");
    }
    Ok(())
}

fn handle_import(path: String) -> anyhow::Result<()> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("Failed to read file '{}': {}", path, e))?;
    let imported: Store = toml::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse import file '{}': {}", path, e))?;

    let local = Store::load()?;
    let new_count = imported
        .commands
        .iter()
        .filter(|c| !local.commands.iter().any(|lc| lc.id == c.id))
        .count();
    let new_folders = imported
        .folders
        .iter()
        .filter(|f| !local.folders.iter().any(|lf| lf.id == f.id))
        .count();

    println!(
        "Import preview: {} new commands, {} new folders",
        new_count, new_folders
    );

    let answer = prompt("Import? [y/N] ")?;
    if answer.to_lowercase() == "y" {
        imported.save_to(std::path::Path::new(
            &dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
                .join(".sac")
                .join("commands.toml"),
        ))?;
        println!("Import complete.");
    } else {
        println!("Import cancelled.");
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    // Daily auto-check
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    if config.general.auto_check_remote && config.general.last_check != today {
        if config.commands_source.mode == "remote" && !config.commands_source.url.is_empty() {
            let store = Store::load()?;
            let _ = sync::sync_check(&config, &store);
        }
        config.general.last_check = today;
        config.save()?;
    }

    match cli.command {
        None => {
            let store = Store::load()?;
            let output = tui::run_tui(store)?;
            if let Some(cmd) = output {
                print!("{}", cmd);
            }
        }
        Some(Commands::Add { folder }) => handle_add(folder)?,
        Some(Commands::NewFolder { name, parent }) => handle_new_folder(name, parent)?,
        Some(Commands::Edit { command_id }) => handle_edit(command_id)?,
        Some(Commands::Delete { command_id }) => handle_delete(command_id)?,
        Some(Commands::Sync { force }) => {
            let store = Store::load()?;
            if force {
                eprint!("Force sync will overwrite local changes. Continue? [y/N] ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() == "y" {
                    sync::sync_check(&config, &store)?;
                }
            } else {
                sync::sync_check(&config, &store)?;
            }
        }
        Some(Commands::Config(args)) => match args.subcommand {
            None => {
                let toml_str = toml::to_string_pretty(&config)?;
                println!("{}", toml_str);
            }
            Some(ConfigSubcommand::Set { key, value }) => {
                config.set(&key, &value)?;
                config.save()?;
                println!("Config updated: {} = {}", key, value);
            }
        },
        Some(Commands::Where { target }) => match target {
            WhereTarget::Config => {
                let path = dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?
                    .join(".sac/config.toml");
                println!("{}", path.display());
            }
            WhereTarget::Commands => {
                let path = dirs::home_dir()
                    .ok_or_else(|| anyhow::anyhow!("cannot find home directory"))?
                    .join(".sac/commands.toml");
                println!("{}", path.display());
            }
        },
        Some(Commands::Install) => {
            shell::install()?;
        }
        Some(Commands::Export { path }) => {
            let store = Store::load()?;
            store.save_to(std::path::Path::new(&path))?;
            println!("Exported to {}", path);
        }
        Some(Commands::Import { path }) => handle_import(path)?,
    }
    Ok(())
}
