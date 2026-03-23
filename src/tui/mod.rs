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
    // Open /dev/tty directly as the rendering backend.
    //
    // Why not stdout or stderr?
    // The shell integration does: result=$(command sac "$@" 2>/dev/tty)
    //   - stdout → pipe captured by $()  →  crossterm's TIOCGWINSZ on stdout fails
    //   - stderr → redirected to /dev/tty, but not all paths are guaranteed
    // Opening /dev/tty explicitly is the industry-standard approach used by fzf, vim, etc.
    // /dev/tty always resolves to the process's controlling terminal regardless of any
    // stdout/stderr redirections, so Terminal::new() gets the correct terminal size and
    // all escape sequences reach the real screen.
    //
    // Key events are still read from stdin, which is always inherited from the parent shell
    // (command substitution does not redirect stdin).
    let tty = std::fs::OpenOptions::new()
        .write(true)
        .open("/dev/tty")
        .context("Cannot open /dev/tty — sac requires an interactive terminal")?;

    enable_raw_mode()?;

    let backend = CrosstermBackend::new(BufWriter::new(tty));
    let mut terminal = Terminal::new(backend)?;
    execute!(terminal.backend_mut(), EnterAlternateScreen)?;

    let mut app = App::new(store);
    let run_result = run_loop(&mut terminal, &mut app);

    // Always restore terminal state, even if run_loop returned an error.
    // Use let _ = so that every cleanup step runs regardless of earlier failures.
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
                // Only handle Press and Repeat; ignore Release to avoid duplicate events.
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
