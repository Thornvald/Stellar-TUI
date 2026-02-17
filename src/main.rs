mod app;
mod build;
mod config;
mod engine;
mod input;
mod notify;
mod types;
mod ui;

use app::App;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

const TICK_RATE: Duration = Duration::from_millis(33); // ~30 fps

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = run_app(&mut terminal).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    let mut last_tick = Instant::now();

    loop {
        // Keep build output draining at high frequency for smooth log updates.
        app.poll_build();

        // Render
        terminal.draw(|f| ui::draw(f, &app))?;

        // Poll for events with timeout to maintain tick rate
        let mut timeout = TICK_RATE
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if app.build_state == crate::types::BuildState::Running {
            timeout = timeout.min(Duration::from_millis(5));
        }

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                // Only handle key press events, ignore release/repeat
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Ctrl+C always quits
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    app.should_quit = true;
                }

                input::handle_key(&mut app, key);
            }
        }

        // Tick update
        if last_tick.elapsed() >= TICK_RATE {
            app.tick = app.tick.wrapping_add(1);
            last_tick = Instant::now();
        }

        if app.should_quit {
            // Cancel running build before quitting
            app.cancel_build();
            break;
        }
    }

    Ok(())
}
