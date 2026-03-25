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
            "  if [[ $# -eq 0 ]]; then\n",
            "    local tmp result\n",
            "    tmp=$(mktemp) || return 1\n",
            "    command sac >\"$tmp\" 2>/dev/tty\n",
            "    { IFS='' read -r -d '' result; } < \"$tmp\"; result=${result%$'\\n'}\n",
            "    rm -f -- \"$tmp\"\n",
            "    [[ -z \"$result\" ]] && return\n",
            "    if zle; then\n",
            "      LBUFFER=$result\n",
            "      RBUFFER=''\n",
            "      zle reset-prompt\n",
            "    else\n",
            "      print -rz -- \"$result\"\n",
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
            "    { IFS='' read -r -d '' result; } < \"$tmp\"; result=${result%$'\\n'}\n",
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

// Old snippets used by previous versions — detected and auto-upgraded by sac install.
const OLD_ZSH_BASH_SNIPPET: &str = concat!(
    "\n# sac shell integration\n",
    "function sac() { local result; result=$(command sac \"$@\" 2>/dev/tty); ",
    "if [[ -n \"$result\" ]]; then BUFFER=\"$result\"; zle redisplay; fi }\n",
);

// v0.1.5 zsh snippet (used $(<file) + BUFFER + zle redisplay — loses backslashes)
const OLD_ZSH_V015_SNIPPET: &str = concat!(
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
);

const OLD_FISH_SNIPPET: &str = concat!(
    "\n# sac shell integration\n",
    "function sac\n",
    "    set result (command sac $argv 2>/dev/tty)\n",
    "    if test -n \"$result\"\n",
    "        commandline \"$result\"\n",
    "    end\n",
    "end\n",
);

/// Remove the old (v0.1.x) integration snippet via exact string replacement.
/// Returns the cleaned content. If neither old snippet is found verbatim,
/// returns the content unchanged (caller will print manual instructions).
fn strip_old_integration(content: &str) -> String {
    content
        .replace(OLD_ZSH_BASH_SNIPPET, "")
        .replace(OLD_ZSH_V015_SNIPPET, "")
        .replace(OLD_FISH_SNIPPET, "")
}

/// Write the shell integration to the rc file.
/// - If the new format is already present (end marker found): skip.
/// - If the old format is present: auto-upgrade (remove old, append new).
/// - If old format cannot be removed verbatim: print manual instructions.
/// - Otherwise: append the new snippet.
pub fn write_integration(shell_type: &ShellType) -> Result<()> {
    let rc_path = get_rc_path(shell_type)?;

    let existing = if rc_path.exists() {
        std::fs::read_to_string(&rc_path)
            .with_context(|| format!("Failed to read {}", rc_path.display()))?
    } else {
        String::new()
    };

    // Already up to date: must have the end marker AND the read-r signature of the current version.
    if existing.contains("# end sac shell integration") && existing.contains("read -r -d ''") {
        println!("Shell integration already installed in {}", rc_path.display());
        return Ok(());
    }

    let new_content = if existing.contains("sac shell integration") {
        // Old format detected — try to auto-upgrade
        let cleaned = strip_old_integration(&existing);
        if cleaned.contains("sac shell integration") {
            // Could not remove verbatim (user edited the block)
            println!("Old sac shell integration found in {}.", rc_path.display());
            println!("Please manually remove the block between");
            println!("  '# sac shell integration'  and  the closing '}}' or 'end',");
            println!("then run `sac install` again.");
            return Ok(());
        }
        println!(
            "Upgrading sac shell integration in {} ...",
            rc_path.display()
        );
        let mut s = cleaned;
        s.push_str(integration_snippet(shell_type));
        s
    } else {
        let mut s = existing;
        s.push_str(integration_snippet(shell_type));
        s
    };

    if let Some(parent) = rc_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    std::fs::write(&rc_path, new_content)
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
