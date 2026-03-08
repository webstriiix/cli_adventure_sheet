use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_notes_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('e') | KeyCode::Enter => {
            // Enter edit mode
            let raw = app
                .active_character
                .as_ref()
                .and_then(|c| c.notes.clone())
                .unwrap_or_default();
            // Strip internal [SKILLS:...] tag from edit buffer
            app.notes_buffer = if let Some(start) = raw.find("[SKILLS:") {
                let end = raw[start..].find(']').map(|e| start + e + 1).unwrap_or(raw.len());
                format!("{}{}", &raw[..start], &raw[end..]).trim().to_string()
            } else {
                raw
            };
            app.notes_cursor = app.notes_buffer.len();
            app.editing_notes = true;
            app.status_msg.clear();
        }
        KeyCode::Up => {
            if app.content_scroll > 0 {
                app.content_scroll -= 1;
            }
        }
        KeyCode::Down => app.content_scroll += 1,
        _ => {}
    }
}

pub fn handle_notes_edit_key(app: &mut App, key: KeyEvent) {
    let buf = &mut app.notes_buffer;
    let mut cursor = app.notes_cursor;

    match key.code {
        KeyCode::Esc => {
            app.save_notes();
            app.editing_notes = false;
        }
        KeyCode::Left => {
            if cursor > 0 {
                // Find previous char boundary
                cursor -= 1;
                while !buf.is_char_boundary(cursor) {
                    cursor -= 1;
                }
            }
        }
        KeyCode::Right => {
            if cursor < buf.len() {
                // Find next char boundary
                cursor += 1;
                while !buf.is_char_boundary(cursor) {
                    cursor += 1;
                }
            }
        }
        KeyCode::Up => {
            // Very naive up: find start of current line, then back up to prev line start
            let curr_line_start = buf[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
            if curr_line_start > 0 {
                let prev_line_start = buf[..curr_line_start - 1]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let col = cursor - curr_line_start;
                let prev_line_len = curr_line_start - 1 - prev_line_start;
                cursor = prev_line_start + col.min(prev_line_len);
                while cursor > 0 && !buf.is_char_boundary(cursor) {
                    cursor -= 1;
                }
            }
        }
        KeyCode::Down => {
            let curr_line_start = buf[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let col = cursor - curr_line_start;
            if let Some(next_line_start) = buf[cursor..].find('\n').map(|i| cursor + i + 1) {
                let next_line_len = buf[next_line_start..]
                    .find('\n')
                    .unwrap_or(buf.len() - next_line_start);
                cursor = next_line_start + col.min(next_line_len);
                while cursor > 0 && !buf.is_char_boundary(cursor) {
                    cursor -= 1;
                }
            }
        }
        KeyCode::Home => {
            cursor = buf[..cursor].rfind('\n').map(|i| i + 1).unwrap_or(0);
        }
        KeyCode::End => {
            cursor = buf[cursor..]
                .find('\n')
                .map(|i| cursor + i)
                .unwrap_or(buf.len());
        }
        KeyCode::Enter => {
            buf.insert(cursor, '\n');
            cursor += 1;
        }
        KeyCode::Backspace => {
            if cursor > 0 {
                let mut prev = cursor - 1;
                while !buf.is_char_boundary(prev) {
                    prev -= 1;
                }
                buf.remove(prev);
                cursor = prev;
            }
        }
        KeyCode::Delete => {
            if cursor < buf.len() {
                buf.remove(cursor);
            }
        }
        KeyCode::Char(c) => {
            buf.insert(cursor, c);
            cursor += c.len_utf8();
        }
        _ => {}
    }
    app.notes_cursor = cursor;
}
