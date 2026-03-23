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
    // The new shell integration runs sac in the foreground via a tmpfile:
    //   command sac >"$tmp" 2>/dev/tty
    //
    // The process is NOT inside $(), so ZLE does not intercept stdin.
    // stdin is inherited from the interactive shell and is a real TTY.
    // We only need to open /dev/tty for the rendering backend (writes) so
    // that the TUI draws directly to the terminal regardless of stdout/stderr
    // redirections (the tmpfile redirect makes stdout a pipe).
    let tty_file = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .context("Cannot open /dev/tty — sac requires an interactive terminal")?;

    enable_raw_mode()?;

    // Write TUI output to /dev/tty; reads (key events) come from stdin which
    // is already the real TTY when running in the foreground.
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
