use anyhow::{Context, Result};
use std::path::PathBuf;

pub enum ShellType {
    Zsh,
    Bash,
    Fish,
}

/// Read $SHELL env var, detect zsh/bash/fish, default to Bash.
pub fn detect_shell() -> ShellType {
    match std::env::var("SHELL").unwrap_or_default().as_str() {
        s if s.contains("zsh") => ShellType::Zsh,
        s if s.contains("fish") => ShellType::Fish,
        _ => ShellType::Bash,
    }
}

/// Return the rc file path for the given shell type.
pub fn get_rc_path(shell_type: &ShellType) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Cannot find home directory")?;
    let path = match shell_type {
        ShellType::Zsh => home.join(".zshrc"),
        ShellType::Bash => home.join(".bashrc"),
        ShellType::Fish => home.join(".config").join("fish").join("config.fish"),
    };
    Ok(path)
}

fn integration_snippet(shell_type: &ShellType) -> &'static str {
    match shell_type {
        ShellType::Zsh | ShellType::Bash => {
            concat!(
                "\n# sac shell integration\n",
                "function sac() { local result; result=$(command sac \"$@\" 2>/dev/tty); if [[ -n \"$result\" ]]; then BUFFER=\"$result\"; zle redisplay; fi }\n"
            )
        }
        ShellType::Fish => {
            concat!(
                "\n# sac shell integration\n",
                "function sac\n",
                "    set result (command sac $argv 2>/dev/tty)\n",
                "    if test -n \"$result\"\n",
                "        commandline \"$result\"\n",
                "    end\n",
                "end\n"
            )
        }
    }
}

/// Check if "sac shell integration" already exists in rc file; if not, append the snippet.
pub fn write_integration(shell_type: &ShellType) -> Result<()> {
    let rc_path = get_rc_path(shell_type)?;

    let existing = if rc_path.exists() {
        std::fs::read_to_string(&rc_path)
            .with_context(|| format!("Failed to read {}", rc_path.display()))?
    } else {
        String::new()
    };

    if existing.contains("sac shell integration") {
        println!("Shell integration already installed in {}", rc_path.display());
        return Ok(());
    }

    let snippet = integration_snippet(shell_type);
    let mut content = existing;
    content.push_str(snippet);

    if let Some(parent) = rc_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(&rc_path, content)
        .with_context(|| format!("Failed to write {}", rc_path.display()))?;

    println!("Shell integration installed in {}", rc_path.display());
    println!("Restart your shell or run: source {}", rc_path.display());
    Ok(())
}

/// Detect shell, get rc path, write integration.
pub fn install() -> Result<()> {
    let shell_type = detect_shell();
    write_integration(&shell_type)
}
