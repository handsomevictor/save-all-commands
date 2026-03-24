# Project Structure

```
save-all-commands/
├── Cargo.toml              # Dependency manifest and package metadata
├── commands.toml.example   # Bundled example command library (~100 commands)
├── src/
│   ├── lib.rs              # Library crate root; re-exports all modules
│   ├── main.rs             # Binary entry point; CLI parsing and sub-command dispatch
│   ├── cli.rs              # clap derive-macro definitions for all sub-commands and flags
│   ├── config.rs           # Config file read/write (~/.sac/config.toml)
│   ├── store.rs            # Command store read/write (~/.sac/commands.toml)
│   ├── search.rs           # Search engine (fuzzy + exact + weighted scoring)
│   ├── sync.rs             # Remote HTTP sync (download, diff, user-confirmed write)
│   ├── shell.rs            # Shell integration generator (zsh / bash / fish)
│   └── tui/
│       ├── mod.rs          # TUI module root; exports run_tui()
│       ├── app.rs          # App state machine (Mode, BrowseItem, keyboard event handling)
│       └── ui.rs           # ratatui rendering (layout, tables, search box, status bar)
├── tests/
│   ├── test_store.rs       # Store CRUD, query, and validation tests
│   ├── test_config.rs      # Config read/write and set() method tests
│   ├── test_validation.rs  # Constraint enforcement (folder/command count limits)
│   ├── test_search.rs      # Fuzzy search, exact search, Unicode, empty-query tests
│   ├── test_cli.rs         # CLI sub-command parsing tests
│   └── test_sync.rs        # Diff logic and format validation tests
└── docs/
    ├── README_CN.md        # Chinese README
    ├── PROGRESS.md         # Changelog (English)
    ├── PROGRESS_CN.md      # Changelog (Chinese)
    ├── STRUCTURE.md        # Project structure (this file, English)
    ├── STRUCTURE_CN.md     # Project structure (Chinese)
    ├── LESSON_LEARNED.md   # Development bug log (English)
    ├── LESSON_LEARNED_CN.md # Development bug log (Chinese)
    ├── TUTORIAL.md         # Full feature reference (English)
    ├── TUTORIAL_CN.md      # Full feature reference (Chinese)
    ├── COMMANDS.md         # CLI quick-reference (English)
    ├── COMMANDS_CN.md      # CLI quick-reference (Chinese)
    └── assets/
        └── screenshot.png  # TUI interface screenshot
```

---

## Module Reference

### `src/main.rs`

Binary entry point. Parses command-line arguments via clap and dispatches to the appropriate handler function. Invoked with no sub-command, starts the TUI.

### `src/cli.rs`

Defines all sub-command structs and argument types using clap's derive macros. Provides type-safe, self-documenting CLI parsing with automatic `--help` generation.

### `src/config.rs`

Manages `~/.sac/config.toml`. Exposes a `Config` struct with typed fields for all settings. Supports reading and updating individual values by dot-notation key path (e.g. `general.auto_check_remote`).

### `src/store.rs`

Manages `~/.sac/commands.toml`. The `Store` struct holds a flat list of `Folder` and `Command` records and provides methods for:
- Tree traversal (`children_folders`, `folder_commands`, `breadcrumb`)
- Structural validation (combined per-folder limit of 10)
- Auto-repair of duplicate command IDs (`auto_fix_ids`)

### `src/search.rs`

Implements two search modes via `Searcher`:

- **Fuzzy search** (`fuzzy_search`) — uses [nucleo-matcher](https://github.com/helix-editor/nucleo) for weighted fuzzy scoring across `cmd`, `desc`, `comment`, and `tags`. Results are ranked by priority tiers: tag match → cmd exact → desc exact → comment exact → fuzzy score. Tie-breaks use `last_used` timestamp, then command ID.
- **Exact search** (`exact_search`) — triggered by the `//` prefix; returns commands whose combined haystack contains the query as a literal substring.

### `src/sync.rs`

Fetches a remote TOML file via HTTP GET, parses it as a `Store`, computes a human-readable diff against the local store, displays added/modified/removed commands, and writes on user confirmation. Supports `--force` to skip the confirmation prompt.

### `src/shell.rs`

Generates and installs shell integration snippets for zsh, bash, and fish. The integration:
1. Guards TUI entry with an argument count check — sub-commands pass through directly
2. Runs `sac` in the foreground with stdout redirected to a tmpfile (avoids `$()` / ZLE conflict)
3. Reads the tmpfile and writes the result to the shell's input buffer (`BUFFER` in zsh/bash, `commandline` in fish)
4. Detects ZLE context (`if zle`) before calling ZLE builtins to avoid "widgets can only be called when ZLE is active" errors

`write_integration()` detects both the current and legacy snippet formats and performs a clean in-place upgrade.

### `src/tui/mod.rs`

Entry point for the TUI. Opens `/dev/tty` write-only as the ratatui backend (bypasses any stdout/stderr redirection in the shell invocation chain). Runs the main event loop: render → read key event → update app state → repeat. Restores terminal state on exit via `let _ =` wrapped cleanup calls.

### `src/tui/app.rs`

The `App` struct is the central state machine. It owns:
- `mode: Mode` — `Browse` or `Search`
- `search_mode: SearchMode` — `Fuzzy` or `Exact`
- `current_folder`, `breadcrumb` — navigation state
- `items: Vec<BrowseItem>` — current folder contents (folders + commands)
- `search_query`, `search_results`, `search_selected` — search state
- `output: Option<String>` — command to paste when quitting

Key methods: `handle_key()`, `enter_folder()`, `go_back()`, `load_items()`, `confirm_command()`, `refresh_search()`, `effective_query()`.

### `src/tui/ui.rs`

Renders the TUI using ratatui. Layout (top to bottom):

1. **Header** (1 line) — key hint bar
2. **Search box** (3 lines) — query input with mode label
3. **Main panel** (fills remaining space) — Browse table or Search results table
4. **Status bar** (1 line) — item counts or search statistics

Both tables use a three-column layout:

| Column | Width | Content |
|--------|-------|---------|
| Number | 6 chars | `[1]`–`[0]`, blank beyond 10 |
| Command | ~52% of available, clamped 20–52 | `$  <cmd>` truncated with `…` |
| Description | remaining | Word-wrapped desc (max 3 lines) + meta line in search mode |
