# Changelog

## v0.1.6 — 2026-03-23

### Bug Fixes

- **[Critical] `sac install` could not upgrade old shell integration automatically** — `write_integration()` now detects legacy snippets via exact string matching, removes them, and writes the current version in a single step. No manual rc-file editing required; `sac install` handles the full upgrade.
- **[Critical] Searching `/doc` returned no results** — Fixed `effective_query()`: in Fuzzy mode the function now strips a single leading `/`, so the vim-style `/` activator no longer pollutes the query string. `/doc` → actual query `doc`; `//doc` → Exact mode (double-slash behavior unchanged).

---

## v0.1.5 — 2026-03-23

### Bug Fixes

- **[Critical] `sac:zle: widgets can only be called when ZLE is active`** — Complete rewrite of zsh/bash/fish shell integration:
  - Guard TUI entry with `[[ $# -eq 0 ]]`; all other argument forms pass through via `command sac "$@"` (fixes `--version`, `add`, and other sub-commands being incorrectly intercepted).
  - Replace `$()` capture with a tmpfile approach (`command sac >"$tmp" 2>/dev/tty`): sac runs in the foreground, so ZLE cannot intercept stdin.
  - Use `if zle; then ... else print -z -- "$result"; fi` to detect ZLE context before setting `BUFFER` or calling `zle redisplay`.
  - Added `# end sac shell integration` terminator; `sac install` detects old-format snippets and prompts for upgrade.
- **[Critical] `Error: Failed to initialize input reader`** — Removed `dup2(tty_fd, STDIN_FILENO)`. kqueue cannot watch a `/dev/tty` fd substituted via dup2. The new shell integration runs sac in the foreground, so stdin is already a real TTY and dup2 is unnecessary.
- **Removed `libc` dependency** — dup2 approach abandoned; `libc` crate no longer needed.

---

## v0.1.4 — 2026-03-23

### Bug Fixes

- **`--version` flag not working** — `#[command]` was missing the `version` attribute on the clap struct. Added; `sac --version` now outputs the version string correctly.
- **[Critical] TUI frozen (ZLE conflict)** — Opened `/dev/tty` with `O_RDWR` and used `dup2` to redirect stdin (fd 0) to `/dev/tty`. Under zsh's ZLE, `$()` sub-shells hold stdin through ZLE, causing `event::read()` to block indefinitely. After dup2, fd 0 pointed directly at the terminal device, bypassing ZLE.
- **Ctrl+C could not exit the TUI** — Added `KeyModifiers::CONTROL + 'c'` handling in both Browse and Search modes; Ctrl+C now exits immediately without outputting any command.

---

## v0.1.3 — 2026-03-23

### Bug Fixes

- **[Critical] Terminal frozen after running `sac`** — Switched the TUI render backend from `stderr` to a directly-opened `/dev/tty` file handle. All cleanup paths changed to `let _ =` to guarantee terminal state is restored on any exit path.

---

## v0.1.2 — 2026-03-23

### Changes

- **Unified TUI numbering** — Removed the separate folder/command section headers and dividers. Folders and commands share a single numbered list (`1`–`9`, `0`). Selecting a folder navigates into it; selecting a command pastes it into the terminal.
- **Combined per-folder limit** — Each folder may contain at most **10 items total** (subfolders + commands), matching the available key slots. Previously subfolders and commands were counted independently.
- **Auto-repair duplicate IDs on startup** — If `commands.toml` contains duplicate command IDs, `sac` reassigns them in order and prints a warning, then continues normally. Structural errors (exceeding the 10-item limit) still cause a hard exit with an error message.

### New

- `Store::auto_fix_ids()` — detects and repairs duplicate command IDs, returns whether any changes were made.
- New test cases: `test_validate_combined_limit_ok/exceeded`, `test_auto_fix_ids_no_duplicates`, `test_auto_fix_ids_with_duplicates`, `test_auto_fix_ids_all_same` (46 total tests, all passing).

---

## v0.1.1 — 2026-03-23

### Bug Fixes

- **[Critical] Selected command executed directly instead of pasted into input bar** — Switched the TUI render backend from `stdout` to `stderr`. The shell integration's `result=$(command sac "$@" 2>/dev/tty)` then captures only the bare command text on stdout, eliminating the escape-code contamination that caused the command to be executed immediately.

---

## v0.1.0 — 2026-03-23

### Initial Release

- **Data layer** — `Store` (commands.toml read/write), `Config` (config.toml read/write)
- **Search layer** — fuzzy search (nucleo-matcher weighted scoring), exact search (`//` prefix)
- **TUI layer** — Browse mode (tree-based folder navigation), Search mode (real-time filtering)
- **CLI sub-commands** — `add`, `new-folder`, `edit`, `delete`, `sync`, `config`, `where`, `install`, `export`, `import`
- **Shell integration** — zsh / bash / fish support; `sac install` one-command setup
- **Sync layer** — remote HTTP sync, diff display, user-confirmed write
- **Test suite** — 41 test cases, all passing
