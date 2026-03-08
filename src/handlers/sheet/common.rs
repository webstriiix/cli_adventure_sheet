use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_default_content_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Up => {
            if app.content_scroll > 0 {
                app.content_scroll -= 1;
            }
        }
        KeyCode::Down => app.content_scroll += 1,
        _ => {}
    }
}
