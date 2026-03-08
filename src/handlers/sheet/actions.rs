use crate::app::App;
use crate::models::app_state::ActionsSubTab;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_actions_key(app: &mut App, key: KeyEvent) {
    // If a detail modal is open, any key closes it
    if app.actions_detail_modal.is_some() {
        app.actions_detail_modal = None;
        return;
    }

    let on_limited = app.actions_sub_tab == ActionsSubTab::LimitedUse;

    // Check for Shift+K first so it isn't swallowed by regular 'k' (Up)
    if key.code == KeyCode::Char('K')
        || (key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::SHIFT))
    {
        open_action_detail_modal(app);
        return;
    }

    match key.code {
        KeyCode::Esc => app.sidebar_focused = true,
        // Sub-tab navigation always on Left/Right
        KeyCode::Left => {
            app.content_scroll = 0;
            app.actions_sub_tab = match app.actions_sub_tab {
                ActionsSubTab::All => ActionsSubTab::All,
                ActionsSubTab::Attack => ActionsSubTab::All,
                ActionsSubTab::Action => ActionsSubTab::Attack,
                ActionsSubTab::BonusAction => ActionsSubTab::Action,
                ActionsSubTab::Reaction => ActionsSubTab::BonusAction,
                ActionsSubTab::Other => ActionsSubTab::Reaction,
                ActionsSubTab::LimitedUse => ActionsSubTab::Other,
            };
        }
        KeyCode::Right => {
            app.content_scroll = 0;
            app.actions_sub_tab = match app.actions_sub_tab {
                ActionsSubTab::All => ActionsSubTab::Attack,
                ActionsSubTab::Attack => ActionsSubTab::Action,
                ActionsSubTab::Action => ActionsSubTab::BonusAction,
                ActionsSubTab::BonusAction => ActionsSubTab::Reaction,
                ActionsSubTab::Reaction => ActionsSubTab::Other,
                ActionsSubTab::Other => ActionsSubTab::LimitedUse,
                ActionsSubTab::LimitedUse => ActionsSubTab::LimitedUse,
            };
        }
        // Up/Down: row selection in LimitedUse, scroll elsewhere
        KeyCode::Up => {
            if on_limited {
                let len = app
                    .char_actions
                    .as_ref()
                    .map(|a| a.limited_use.len())
                    .unwrap_or(0);
                let cur = app.actions_list_state.selected().unwrap_or(0);
                if len > 0 {
                    let prev = if cur == 0 { len - 1 } else { cur - 1 };
                    app.actions_list_state.select(Some(prev));
                }
            } else if app.content_scroll > 0 {
                app.content_scroll -= 1;
            }
        }
        KeyCode::Down => {
            if on_limited {
                let len = app
                    .char_actions
                    .as_ref()
                    .map(|a| a.limited_use.len())
                    .unwrap_or(0);
                let cur = app.actions_list_state.selected().unwrap_or(0);
                if len > 0 {
                    app.actions_list_state.select(Some((cur + 1) % len));
                }
            } else {
                app.content_scroll += 1;
            }
        }
        // Spend a use with '-'
        KeyCode::Char('-') | KeyCode::Char('_') if on_limited => {
            change_limited_use(app, -1);
        }
        // Recover a use with '+'
        KeyCode::Char('+') | KeyCode::Char('=') if on_limited => {
            change_limited_use(app, 1);
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

/// Adjust the `current_uses` of the currently selected Limited Use action
/// by `delta` (−1 to spend, +1 to recover) and persist it to the backend.
pub fn change_limited_use(app: &mut App, delta: i32) {
    let char_id = match app.active_character.as_ref().map(|c| c.id) {
        Some(id) => id,
        None => return,
    };
    let selected = app.actions_list_state.selected().unwrap_or(0);
    let actions = match app.char_actions.as_mut() {
        Some(a) => a,
        None => return,
    };
    let item = match actions.limited_use.get_mut(selected) {
        Some(i) => i,
        None => return,
    };
    let max = item.max_uses.unwrap_or(0);
    let cur = item.current_uses.unwrap_or(max);
    let new_val = (cur + delta).clamp(0, max);
    item.current_uses = Some(new_val);
    let resource_name = item.name.clone();
    let resource_name_for_async = resource_name.clone();
    let rt = app.rt.clone();
    let client = app.client.clone();
    rt.spawn(async move {
        let _ = client
            .patch_resource_uses(char_id, &resource_name_for_async, new_val)
            .await;
    });
    app.status_msg = format!("{}: {}/{}", resource_name, new_val, max);
}

/// Open the detail modal for the currently selected Limited Use action.
pub fn open_action_detail_modal(app: &mut App) {
    if app.actions_sub_tab != crate::models::app_state::ActionsSubTab::LimitedUse {
        return; // Only implemented for LimitedUse list currently
    }

    let selected = app.actions_list_state.selected().unwrap_or(0);
    if let Some(actions) = &app.char_actions {
        if let Some(item) = actions.limited_use.get(selected) {
            let name = item.name.clone();
            let mut desc = item
                .description
                .clone()
                .unwrap_or_else(|| "No description available.".to_string());

            // Clean up description formatting
            desc = desc.replace("\\n", "\n").replace("\"", "");
            if desc.starts_with('[') && desc.ends_with(']') {
                desc = desc[1..desc.len() - 1].to_string();
            }
            desc = crate::ui::sheet::features::strip_tags(&desc);

            app.actions_detail_modal = Some((name, desc));
        }
    }
}
