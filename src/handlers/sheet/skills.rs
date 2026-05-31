use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

const SKILLS_COUNT: usize = 18;
const SKILLS: [&str; 18] = [
    "Acrobatics", "Animal Handling", "Arcana", "Athletics", "Deception",
    "History", "Insight", "Intimidation", "Investigation", "Medicine",
    "Nature", "Perception", "Performance", "Persuasion", "Religion",
    "Sleight of Hand", "Stealth", "Survival"
];

pub fn handle_skills_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Left => app.sidebar_focused = true,
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Up | KeyCode::Char('k') => {
            if app.selected_list_index > 0 {
                app.selected_list_index -= 1;
            } else {
                app.selected_list_index = SKILLS_COUNT - 1;
            }
            app.sheet_table_state.select(Some(app.selected_list_index));
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.selected_list_index = (app.selected_list_index + 1) % SKILLS_COUNT;
            app.sheet_table_state.select(Some(app.selected_list_index));
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            let skill_name = SKILLS[app.selected_list_index].to_lowercase();
            app.toggle_proficiency("skill", &skill_name);
        }
        _ => {}
    }
}
