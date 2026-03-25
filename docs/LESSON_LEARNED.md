# Development Bug Log

## [2026-03-23] Critical: `event::read()` blocked indefinitely after switching to `/dev/tty` backend

**Symptom:** After switching the render backend to `/dev/tty`, the TUI rendered correctly but no keypress was ever registered. The terminal was completely frozen.

**Root cause:** zsh's ZLE (Zsh Line Editor) owns stdin (fd 0) in an interactive shell. In a `$()` sub-shell, the child process inherits stdin, but ZLE retains control through the process group and terminal ownership. `crossterm::event::read()` reads from stdin; because ZLE intercepts it, that read never returns.

Switching to `/dev/tty` as a _write_ backend only solved the rendering problem, not the read problem.

**Fix:** Following fzf's approach: open `/dev/tty` with `O_RDWR`, then call `libc::dup2(tty_fd, STDIN_FILENO)` to redirect fd 0 to the terminal device directly. After dup2, fd 0 bypasses the ZLE-managed descriptor path, and `event::read()` works normally.

```rust
let tty_file = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
unsafe { libc::dup2(tty_file.as_raw_fd(), libc::STDIN_FILENO) };
enable_raw_mode()?;
let backend = CrosstermBackend::new(BufWriter::new(tty_file));
```

**Lesson:** Inside a `$()` sub-shell under zsh, the child process's stdin is technically inherited from the shell but ZLE intercepts keyboard input at the process-group level. Any TUI tool that needs to read keyboard events in this context must remap stdin to `/dev/tty` via dup2. This is standard practice for fzf, peco, and similar interactive selectors.

---

## [2026-03-23] Critical: Terminal frozen after switching render backend to stderr

**Symptom:** After switching from stdout to stderr as the render backend, running `sac` caused the terminal to freeze immediately with no output. Only `Ctrl+C` could recover it.

**Root cause:** `crossterm::terminal::size()` on Unix calls `ioctl(STDOUT_FILENO, TIOCGWINSZ, ...)` — it queries the window size through stdout, not stderr. The shell integration ran `result=$(command sac "$@" 2>/dev/tty)`, making stdout a pipe rather than a TTY. `TIOCGWINSZ` returned `ENOTTY`, causing `Terminal::new(backend)?` to fail. At that point `enable_raw_mode()` had already been called, but the error-path cleanup (`disable_raw_mode`, `LeaveAlternateScreen`) was skipped due to early return, leaving the terminal locked in raw mode + alternate screen.

**Fix:** Open `/dev/tty` directly as the render backend (`OpenOptions::new().write(true).open("/dev/tty")`). `/dev/tty` always refers to the process's controlling terminal regardless of stdout/stderr redirection. Wrap all cleanup calls in `let _ =` to guarantee terminal restoration on every exit path, including error paths.

**Lesson:** stdout and stderr can be redirected to pipes or files at any point in a shell invocation chain. TUI tools must open `/dev/tty` directly as the render target — this is the industry-standard approach used by fzf, vim, tmux, and others. All terminal cleanup code must be wrapped with `let _ =`; any `?` in a cleanup sequence may skip subsequent restore calls.

---

## [2026-03-23] Design flaw: Dual numbering in Browse mode caused ambiguous key behavior

**Symptom:** TUI displayed independent `[1]`–`[9]` sequences for folders and commands (e.g. folder `[1]` and command `[1]` coexisted on screen). Pressing a number key behaved differently depending on the order of items, which violated user expectations.

**Root cause:** The original design kept separate numbering for folders and commands, requiring users to mentally track "which [1] am I pressing."

**Fix:** Remove section headers and dividers. All items (folders and commands) share a single sequential list with unified numbering `1`–`9`, `0`. Folders still appear before commands, but the sequence is continuous. The per-folder limit was updated to a combined cap of 10 items (subfolders + commands total), matching the 10 available key slots.

---

## [2026-03-23] Critical: Selected command executed directly instead of pasted into input bar

**Symptom:** After configuring shell integration, selecting a command in the TUI caused it to execute immediately rather than appearing in the terminal input bar.

**Root cause:** Shell integration captured sac's output with `result=$(command sac "$@" 2>/dev/tty)`. The TUI was using `io::stdout()` as its ratatui backend, so all escape sequences (`EnterAlternateScreen`, cursor movement, color codes) streamed into stdout and were captured into `result`. Setting `BUFFER` to this contaminated string caused zsh to execute the embedded command text.

**Fix:** Switch the TUI backend to `io::stderr()`. The shell integration already redirected stderr to `/dev/tty` with `2>/dev/tty`, so TUI output continued displaying correctly; stdout then contained only the bare command string.

```rust
// Before
let mut stdout = io::stdout();
execute!(stdout, EnterAlternateScreen)?;
let backend = CrosstermBackend::new(stdout);

// After
let mut stderr = io::stderr();
execute!(stderr, EnterAlternateScreen)?;
let backend = CrosstermBackend::new(stderr);
```

**Lesson:** Any CLI tool that starts a TUI inside a `$(...)` capture context must ensure TUI rendering goes to stderr or `/dev/tty`. stdout must carry only the final machine-readable result; contaminating it with escape sequences causes shells to misinterpret the captured value.

---

## [2026-03-23] `BrowseItem` missing `Clone` trait caused clippy warning

**Symptom:** A hand-written `impl BrowseItem { pub fn clone(...) }` method triggered a `should_implement_trait` clippy warning: the method name collides with `std::clone::Clone::clone`.

**Root cause:** Rust requires that if a method name matches a standard trait method, the type should implement that trait via `derive` or `impl` rather than defining the method in an inherent `impl` block.

**Fix:** Add `#[derive(Clone)]` to `BrowseItem` and remove the hand-written `clone` method.

---

## [2026-03-23] Critical: `dup2` stdin remapping caused `Failed to initialize input reader`

**Symptom:** After remapping stdin to `/dev/tty` via `dup2(tty_fd, STDIN_FILENO)`, crossterm reported `Failed to initialize input reader` and the TUI could not start.

**Root cause:** crossterm on Unix uses mio + kqueue/epoll to register stdin (fd 0) for event watching. After dup2, fd 0 pointed to a `/dev/tty` device file. On certain macOS/BSD versions, kqueue's `EVFILT_READ` filter cannot be registered against a `/dev/tty` fd, causing mio initialization to fail.

**Fix:** Abandon the dup2 approach entirely. The correct fix is at the shell integration layer: change `result=$(command sac ...)` to the tmpfile approach `command sac >"$tmp" 2>/dev/tty`. In this design, sac runs in the foreground outside of `$()`, ZLE does not intercept stdin, and stdin is already a real TTY. No dup2 is needed; crossterm's kqueue registration succeeds normally. The TUI render backend only needs write-only `/dev/tty` access.

**Lesson:** kqueue's ability to watch `/dev/tty` fds is unreliable on macOS. Never use dup2 to force-replace stdin; the correct solution is to ensure the process starts in the right context (foreground, real TTY stdin) from the beginning — a shell integration architecture problem, not a Rust-layer problem.

---

## [2026-03-23] Critical: `$()` capture + ZLE widget context errors

**Symptom:** After `sac install`, every invocation of `source ~/.zshrc` or `sac --version` printed `sac:zle: widgets can only be called when ZLE is active`.

**Root cause:** The old shell integration unconditionally called `zle redisplay` and set `BUFFER`. These operations are only valid inside a ZLE widget context (i.e. while the user is actively editing a command line). `source ~/.zshrc` and direct sub-command invocations are not in ZLE context, so they always triggered the error. Additionally, `result=$(command sac ...)` placed sac in a `$()` sub-shell, where ZLE intercepted stdin via the process group, causing `event::read()` to block indefinitely.

**Fix:** Completely rewrite the shell integration using the tmpfile approach:
- Argument count guard `[[ $# -eq 0 ]]`: only launch TUI for bare `sac` invocations; pass all other arguments through via `command sac "$@"` (fixes `--version`, `add`, etc.)
- `command sac >"$tmp" 2>/dev/tty`: sac runs in the foreground, no `$()` wrapper, ZLE does not intercept stdin
- `if zle; then ... else print -z -- "$result"; fi`: detect ZLE context before calling ZLE builtins; use `print -z` to populate the next-prompt buffer when outside ZLE

**Lesson:** `$()` + ZLE is two independent fatal problems. The correct shell integration architecture: (1) guard TUI entry with an argument count check; (2) use tmpfile instead of `$()`; (3) use `if zle` to detect context before calling ZLE builtins.

---

## [2026-03-25] Critical: `$(<file)` + `emulate -L zsh` strips `\<newline>` in ZLE context

**Symptom:** Commands containing `\<newline>` (line-continuation syntax, e.g. `curl -X POST "..." \`) were pasted into the terminal with all backslashes removed. Commands appeared on one line with no `\` characters.

**Root cause:** Two compounding issues in the zsh snippet:
1. `result=$(<"$tmp")` — even under `emulate -L zsh`, zsh processes `\<newline>` as a line-continuation sequence during command substitution, silently stripping the backslash and joining the lines.
2. `BUFFER=$result` + `zle redisplay` — assigning to `BUFFER` and calling `zle redisplay` can trigger ZLE to re-parse the string content in certain zsh builds, causing additional escape processing.

**Fix:** Four changes applied in combination:
- `{ IFS='' read -r -d '' result; } < "$tmp"` — `read -r` explicitly disables all backslash escape processing at the shell level; `read -d ''` reads until EOF rather than a newline. This is the gold-standard pattern for binary-safe file reading in zsh/bash.
- `LBUFFER=$result; RBUFFER=''` — assign to `LBUFFER`/`RBUFFER` (the left and right portions of the edit buffer) instead of `BUFFER` directly. This bypasses BUFFER-level ZLE re-parsing.
- `zle reset-prompt` instead of `zle redisplay` — `reset-prompt` redraws only the prompt line; it does not trigger ZLE to re-process the buffer contents.
- `print -rz` instead of `print -z` (non-ZLE path) — the `-r` flag disables escape processing in `print`, matching the `read -r` guarantee on the read side.
- Removed `emulate -L zsh` — not needed and creates a side-effect environment that interacts unpredictably with string processing.

```zsh
# Before (broken)
result=$(<"$tmp")
BUFFER=$result
CURSOR=${#BUFFER}
zle redisplay

# After (fixed)
{ IFS='' read -r -d '' result; } < "$tmp"; result=${result%$'\n'}
LBUFFER=$result
RBUFFER=''
zle reset-prompt
```

**Lesson:** For any shell function that reads file contents and sets ZLE buffer state:
- Always use `read -r` to prevent backslash interpretation. `$(<file)` is not safe for arbitrary content.
- Prefer `LBUFFER`/`RBUFFER` over `BUFFER` to avoid ZLE re-parsing side-effects.
- Use `zle reset-prompt` rather than `zle redisplay` when you only want a visual refresh.
- Use `print -rz` (not `print -z`) to place content into the history/next-prompt buffer.
- Test with commands containing `\`, `\\`, `\n`, and `\<space>` — these are the characters most likely to be silently mangled by shell string processing.

---

## [2026-03-23] `Style` implements `Copy` — unnecessary `.clone()` call

**Symptom:** `meta_style.clone()` triggered a `clone_on_copy` clippy warning.

**Root cause:** ratatui's `Style` type implements `Copy`. Direct assignment copies the value; `.clone()` is redundant and misleading.

**Fix:** Remove the `.clone()` call.
