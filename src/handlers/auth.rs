use crate::app::App;
use crate::models::{
    LoginRequest, SignupRequest,
    app_state::{AuthMode, Screen},
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_login_key(app: &mut App, key: KeyEvent) {
    let field_count = if app.auth_mode == AuthMode::Signup {
        3
    } else {
        2
    };

    match key.code {
        KeyCode::Esc => app.should_quit = true,
        KeyCode::F(2) => {
            app.auth_mode = match app.auth_mode {
                AuthMode::Login => AuthMode::Signup,
                AuthMode::Signup => AuthMode::Login,
            };
            app.auth_focus = 0;
            app.status_msg.clear();
        }
        KeyCode::Tab => {
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                app.auth_focus = if app.auth_focus == 0 {
                    field_count - 1
                } else {
                    app.auth_focus - 1
                };
            } else {
                app.auth_focus = (app.auth_focus + 1) % field_count;
            }
        }
        KeyCode::BackTab => {
            app.auth_focus = if app.auth_focus == 0 {
                field_count - 1
            } else {
                app.auth_focus - 1
            };
        }
        KeyCode::Enter => submit_auth(app),
        KeyCode::Backspace => {
            let idx = auth_field_index(app);
            app.auth_fields[idx].pop();
        }
        KeyCode::Char(c) => {
            let idx = auth_field_index(app);
            app.auth_fields[idx].push(c);
        }
        _ => {}
    }
}

pub fn auth_field_index(app: &App) -> usize {
    match app.auth_mode {
        AuthMode::Login => app.auth_focus,
        AuthMode::Signup => match app.auth_focus {
            0 => 2,
            1 => 0,
            2 => 1,
            _ => 0,
        },
    }
}

pub fn submit_auth(app: &mut App) {
    let rt = app.rt.clone();
    let result = match app.auth_mode {
        AuthMode::Login => {
            let req = LoginRequest {
                email: app.auth_fields[0].clone(),
                password: app.auth_fields[1].clone(),
            };
            rt.block_on(app.client.login(&req))
        }
        AuthMode::Signup => {
            let req = SignupRequest {
                username: app.auth_fields[2].clone(),
                email: app.auth_fields[0].clone(),
                password: app.auth_fields[1].clone(),
            };
            rt.block_on(app.client.signup(&req))
        }
    };

    match result {
        Ok(auth) => {
            app.status_msg = format!("Welcome, {}!", auth.user.username);
            
            // Save session for persistent login
            app.storage.save_session(&crate::utils::storage::Session {
                token: Some(auth.token.clone()),
            });

            app.fetch_compendium_data();
            app.fetch_characters();
            app.screen = Screen::CharacterList;
        }
        Err(e) => {
            app.status_msg = format!("Auth failed: {e}");
        }
    }
}
