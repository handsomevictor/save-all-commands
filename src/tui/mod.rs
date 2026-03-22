pub mod app;
pub mod ui;

pub use app::App;

use anyhow::Result;
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use crate::store::Store;

pub fn run_tui(store: Store) -> Result<Option<String>> {
    // Render TUI on stderr, not stdout.
    //
    // The shell integration captures stdout: result=$(command sac "$@" 2>/dev/tty)
    // Stderr is redirected to the real TTY so the TUI is visible on the terminal.
    // Stdout stays clean so only the selected command text reaches the shell function,
    // which then writes it into BUFFER without executing it.
    // If TUI were on stdout, escape codes would pollute the captured result and the
    // shell would attempt to execute garbled text — the fatal bug this fixes.
    enable_raw_mode()?;
    let mut stderr = io::stderr();
    execute!(stderr, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(store);
    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal regardless of error
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result?;

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
                // Skip key release events (only handle press and repeat)
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
