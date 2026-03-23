pub mod app;
pub mod ui;

pub use app::App;

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::BufWriter;

use crate::store::Store;

pub fn run_tui(store: Store) -> Result<Option<String>> {
    // ── Terminal I/O setup ────────────────────────────────────────────────────
    //
    // Problem: the shell integration runs sac inside a command substitution:
    //   result=$(command sac "$@" 2>/dev/tty)
    //
    // Inside $(), zsh's ZLE (line editor) still owns the terminal.  When a
    // child process inherits stdin in that context, ZLE intercepts key events
    // before they reach the child, so crossterm's event::read() blocks forever
    // — the "frozen terminal" symptom.
    //
    // Solution (same as fzf): open /dev/tty with O_RDWR and dup2 it onto
    // stdin (fd 0).  After the dup2, stdin IS the controlling terminal and
    // is no longer subject to ZLE interception.  The same fd is used for the
    // rendering backend so both reads and writes go through the same tty fd.
    //
    // On stdout: stays as-is (a pipe captured by $()).  After the TUI exits,
    // main() writes the selected command to stdout — the only thing the shell
    // function captures.
    let tty_file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .context("Cannot open /dev/tty — sac requires an interactive terminal")?;

    // Redirect stdin → /dev/tty so crossterm event::read() receives key events
    // directly from the terminal, bypassing any ZLE / job-control interception.
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let tty_fd = tty_file.as_raw_fd();
        let ret = unsafe { libc::dup2(tty_fd, libc::STDIN_FILENO) };
        if ret == -1 {
            return Err(anyhow::anyhow!(
                "dup2(/dev/tty, stdin) failed: {}",
                std::io::Error::last_os_error()
            ));
        }
    }

    enable_raw_mode()?;

    // Use the same /dev/tty fd for the rendering backend.
    // BufWriter batches writes so ratatui's full-frame redraws are efficient.
    let backend = CrosstermBackend::new(BufWriter::new(tty_file));
    let mut terminal = Terminal::new(backend)?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    let mut app = App::new(store);
    let run_result = run_loop(&mut terminal, &mut app);

    // Always restore terminal state regardless of any error in run_loop.
    // Use let _ = so that a failure in one step never skips the remaining ones.
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
    let _ = terminal.show_cursor();
    let _ = disable_raw_mode();

    run_result?;
    Ok(app.output)
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                // Only handle Press and Repeat; ignore Release to avoid duplicates.
                if key_event.kind == crossterm::event::KeyEventKind::Press
                    || key_event.kind == crossterm::event::KeyEventKind::Repeat
                {
                    app.handle_key(key_event)?;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
