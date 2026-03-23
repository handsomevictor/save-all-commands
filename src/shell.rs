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
        ShellType::Zsh => concat!(
            "\n# sac shell integration — do not edit this block\n",
            "function sac() {\n",
            "  emulate -L zsh\n",
            "  if [[ $# -eq 0 ]]; then\n",
            "    local tmp\n",
            "    tmp=$(mktemp) || return 1\n",
            "    command sac >\"$tmp\" 2>/dev/tty\n",
            "    local result\n",
            "    result=$(<\"$tmp\")\n",
            "    rm -f -- \"$tmp\"\n",
            "    [[ -z \"$result\" ]] && return\n",
            "    if zle; then\n",
            "      BUFFER=$result\n",
            "      CURSOR=${#BUFFER}\n",
            "      zle redisplay\n",
            "    else\n",
            "      print -z -- \"$result\"\n",
            "    fi\n",
            "  else\n",
            "    command sac \"$@\"\n",
            "  fi\n",
            "}\n",
            "# end sac shell integration\n",
        ),
        ShellType::Bash => concat!(
            "\n# sac shell integration — do not edit this block\n",
            "function sac() {\n",
            "  if [[ $# -eq 0 ]]; then\n",
            "    local tmp result\n",
            "    tmp=$(mktemp) || return 1\n",
            "    command sac >\"$tmp\" 2>/dev/tty\n",
            "    result=$(<\"$tmp\")\n",
            "    rm -f -- \"$tmp\"\n",
            "    [[ -z \"$result\" ]] && return\n",
            "    READLINE_LINE=$result\n",
            "    READLINE_POINT=${#READLINE_LINE}\n",
            "  else\n",
            "    command sac \"$@\"\n",
            "  fi\n",
            "}\n",
            "# end sac shell integration\n",
        ),
        ShellType::Fish => concat!(
            "\n# sac shell integration — do not edit this block\n",
            "function sac\n",
            "  if test (count $argv) -eq 0\n",
            "    set tmp (mktemp); or return 1\n",
            "    command sac >$tmp 2>/dev/tty\n",
            "    set result (cat $tmp)\n",
            "    rm -f -- $tmp\n",
            "    test -z \"$result\"; and return\n",
            "    commandline -- $result\n",
            "    commandline -f repaint\n",
            "  else\n",
            "    command sac $argv\n",
            "  end\n",
            "end\n",
            "# end sac shell integration\n",
        ),
    }
}

/// Check if "sac shell integration" already exists in rc file.
/// If the old format is detected (no "end sac shell integration" marker),
/// print upgrade instructions and skip writing.
/// If already up to date, skip. Otherwise append the new snippet.
pub fn write_integration(shell_type: &ShellType) -> Result<()> {
    let rc_path = get_rc_path(shell_type)?;

    let existing = if rc_path.exists() {
        std::fs::read_to_string(&rc_path)
            .with_context(|| format!("Failed to read {}", rc_path.display()))?
    } else {
        String::new()
    };

    if existing.contains("# end sac shell integration") {
        println!("Shell integration already installed in {}", rc_path.display());
        return Ok(());
    }

    if existing.contains("sac shell integration") {
        // Old format detected — guide the user to upgrade manually
        println!("Old sac shell integration detected in {}.", rc_path.display());
        println!("Please remove the old block (between '# sac shell integration' and the");
        println!("closing 'end' / closing brace), then run `sac install` again.");
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
