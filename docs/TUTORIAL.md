# Full Feature Reference

## 1. Installation

### Prerequisites

Install the Rust toolchain via [rustup](https://rustup.rs/) if you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build and Install

```bash
git clone https://github.com/handsomevictor/save-all-commands.git
cd save-all-commands
cargo install --path .
```

`sac` is installed to `~/.cargo/bin/sac`. Ensure `~/.cargo/bin` is in your `PATH`.

### First Run

```bash
sac
```

On first launch, `sac` auto-creates `~/.sac/` with default config and an empty commands file. To start with example data:

```bash
mkdir -p ~/.sac
cp /path/to/save-all-commands/commands.toml.example ~/.sac/commands.toml
```

---

## 2. Shell Integration

Shell integration is what makes `sac` useful. Without it, selecting a command only prints it to stdout. With it, the command is pasted directly into your terminal input bar for you to review and run.

### Install

```bash
sac install
```

`sac` will write the appropriate snippet to your shell's rc file and display the path. Then reload:

```bash
source ~/.zshrc         # zsh
source ~/.bashrc        # bash
source ~/.config/fish/config.fish   # fish
```

### How it works

The integration uses a tmpfile approach to avoid the ZLE conflict that arises with `$()` sub-shells:

```bash
# Simplified zsh integration
sac() {
  if [[ $# -eq 0 ]]; then
    local tmp
    tmp=$(mktemp)
    command sac >"$tmp" 2>/dev/tty
    local result
    result=$(<"$tmp")
    rm -f "$tmp"
    if [[ -n "$result" ]]; then
      if zle; then
        BUFFER=$result
        CURSOR=${#BUFFER}
        zle redisplay
      else
        print -z -- "$result"
      fi
    fi
  else
    command sac "$@"
  fi
}
```

Key properties:
- `sac` runs in the **foreground** (no `$()`), so ZLE does not intercept stdin
- Stdout is redirected to a tmpfile; TUI rendering goes to `/dev/tty`
- ZLE context is detected with `if zle` before setting `BUFFER`
- Sub-commands (`sac add`, `sac --version`, etc.) pass through unmodified

### Upgrading the integration

If you previously installed an older version, `sac install` automatically detects and replaces the legacy snippet.

---

## 3. TUI Usage

### Launching

```bash
sac
```

The TUI has two modes: **Browse** and **Search**.

### Browse Mode

The default mode. Displays the current folder's contents as a numbered list: subfolders first (shown in cyan with a 📁 icon), then commands.

| Key | Action |
|-----|--------|
| `1`–`9` / `0` | Navigate directly to the item at that position |
| `↑` / `↓` | Move cursor |
| `Enter` | Enter folder or select command |
| `q` / `Esc` | Go up one folder level; exit at root |
| Any printable character | Append to search query, enter Search mode |
| `Ctrl+C` | Exit without output |

### Search Mode

Activated automatically as soon as you type. All commands are searched in real time across `cmd`, `desc`, `comment`, and `tags`.

| Key | Action |
|-----|--------|
| Continue typing | Append characters, update results live |
| `1`–`9` / `0` | Immediately select the result at that position |
| `↑` / `↓` | Move cursor through results |
| `Enter` | Select highlighted result |
| `Backspace` | Delete last character (Unicode-safe) |
| `Esc` | Clear query, return to Browse mode |
| `Ctrl+C` | Exit without output |

### Vim-style activation

Typing `/` as the first character activates Search mode using vim conventions. The `/` is stripped from the actual query, so `/doc` searches for `doc`. To reach Exact mode, type `//query`.

### Exact Search (`//` prefix)

Prefixing your query with `//` switches to exact substring matching — no fuzzy scoring. Only commands whose combined text (`cmd + desc + comment + tags`) contains the literal string are returned.

```
//kubectl exec    →  returns only commands containing "kubectl exec" verbatim
```

### Selecting a Command

Press the number key or Enter on a highlighted result. The command text is written to your shell's input buffer. You can then:
- Replace `{placeholder}` values with real arguments
- Add or remove flags
- Press Enter to execute — or press `Ctrl+C` to discard

`sac` itself never executes any command.

---

## 4. Managing Commands

### Add a command

```bash
sac add                          # prompt for folder, cmd, desc, comment, tags
sac add --folder <folder-id>     # pre-select a folder
```

### Edit a command

```bash
sac edit <command-id>
```

Opens an interactive prompt with the current field values pre-filled. Press Enter to keep a field unchanged.

### Delete a command

```bash
sac delete <command-id>
```

Prompts for confirmation before deleting.

---

## 5. Managing Folders

### Create a folder

```bash
sac new-folder <name>                        # create at root level
sac new-folder <name> --parent <folder-id>  # create as sub-folder
```

### Per-folder limit

Each folder may contain at most **10 items total** (subfolders + commands). This matches the `1`–`9` / `0` TUI key layout. Operations that would exceed this limit are rejected with an error.

### Finding folder IDs

Folder IDs use dot-notation hierarchy. The root folder has `id = ""`. Browse the TUI or run `sac where commands` to open the file and read the IDs directly.

---

## 6. Configuration

### View current configuration

```bash
sac config
```

Displays the full contents of `~/.sac/config.toml`.

### Modify a value

```bash
sac config set <key> <value>
```

Supported keys:

| Key | Description | Values |
|-----|-------------|--------|
| `general.auto_check_remote` | Auto-check remote on daily first launch | `true` / `false` |
| `commands_source.mode` | Command source | `local` / `remote` |
| `commands_source.path` | Local file path | any path (supports `~`) |
| `commands_source.url` | Remote TOML URL | any HTTP/HTTPS URL |
| `shell.type` | Shell type for integration | `zsh` / `bash` / `fish` |

### Examples

```bash
# Switch to remote mode and set a GitHub Gist URL
sac config set commands_source.mode remote
sac config set commands_source.url https://gist.githubusercontent.com/yourname/xxx/raw/commands.toml

# Disable auto-checking on startup
sac config set general.auto_check_remote false

# Change shell type
sac config set shell.type bash
```

---

## 7. File Paths

```bash
sac where config     # print path to config.toml
sac where commands   # print path to commands.toml
```

Both paths are expanded (no `~` shorthand). Use these for scripting, backups, or opening the files in your editor.

---

## 8. Import and Export

### Export

```bash
sac export ~/backup/commands.toml
```

Copies the current commands file to the specified path.

### Import

```bash
sac import ~/backup/commands.toml
```

Validates the file format, then replaces the current commands file. Invalid files are rejected before any write occurs.

---

## 9. Remote Sync

### Manual sync

```bash
sac sync
```

1. Fetches the TOML file from `commands_source.url`
2. Parses and validates it
3. Computes a diff against local data
4. Displays added, modified, and removed commands
5. Prompts for confirmation
6. Writes the remote data on confirmation

### Force sync

```bash
sac sync --force
```

Skips the confirmation prompt and overwrites local data immediately. Use with caution — any local changes not in the remote will be lost.

### Auto-check on startup

When `general.auto_check_remote = true` (the default) and `commands_source.mode = "remote"`, `sac` checks for remote updates once per day on first launch. If no network is available, the check is silently skipped.

---

## 10. commands.toml Format Reference

### Folder fields

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `id` | string | ✅ | Unique identifier; use dot-notation for hierarchy (`devops.k8s.debug`) |
| `parent` | string | ✅ | Parent folder `id`; root folders use `""` |
| `name` | string | ✅ | Display name shown in the TUI |

### Command fields

| Field | Type | Required | Description |
|-------|------|:--------:|-------------|
| `id` | integer | ✅ | Unique numeric ID; recommended to increment from 1 |
| `folder` | string | ✅ | `id` of the containing folder |
| `cmd` | string | ✅ | Command text; use `{placeholder}` for variable parts |
| `desc` | string | ✅ | Short description shown in the TUI; included in search |
| `comment` | string | | Extended notes; searchable but not shown in the main list |
| `tags` | string[] | | Search tags; tag matches receive the highest search priority |
| `last_used` | string | | ISO 8601 timestamp; auto-managed by `sac` |

### Multi-line commands

Use TOML literal strings (triple single-quotes) for commands that span multiple lines. Backslashes are preserved literally:

```toml
cmd = '''
aws s3 sync s3://{bucket}/{prefix}/ . \
  --exclude "*.tmp" \
  --delete
'''
```

### Minimal template

```toml
[[folders]]
id     = "my-folder"
parent = ""
name   = "My Commands"

[[commands]]
id        = 1
folder    = "my-folder"
cmd       = "echo hello"
desc      = "Print hello"
comment   = ""
tags      = []
last_used = ""
```
