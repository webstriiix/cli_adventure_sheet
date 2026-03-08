mod app;
mod client;
mod models;
mod ui;

use std::io;

use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

use app::App;
use client::ApiClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a tokio runtime for async API calls (separate from the main thread)
    let rt = tokio::runtime::Runtime::new()?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with the runtime handle
    let client = ApiClient::new();
    let mut app = App::new(client, rt.handle().clone());

    // Run loop
    let result = run_loop(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        let event = crossterm::event::read()?;
        app.handle_event(event);

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
pub mod utils;
pub mod handlers;
