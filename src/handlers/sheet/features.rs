use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_features_key(app: &mut App, key: KeyEvent) {
    use crate::app::FeaturesSubTab;
    match key.code {
        KeyCode::Esc => app.sidebar_focused = true,
        KeyCode::Left => {
            app.content_scroll = 0;
            app.features_sub_tab = match app.features_sub_tab {
                FeaturesSubTab::All => FeaturesSubTab::All,
                FeaturesSubTab::ClassFeatures => FeaturesSubTab::All,
                FeaturesSubTab::SpeciesTraits => FeaturesSubTab::ClassFeatures,
                FeaturesSubTab::Feats => FeaturesSubTab::SpeciesTraits,
            };
        }
        KeyCode::Right => {
            app.content_scroll = 0;
            app.features_sub_tab = match app.features_sub_tab {
                FeaturesSubTab::All => FeaturesSubTab::ClassFeatures,
                FeaturesSubTab::ClassFeatures => FeaturesSubTab::SpeciesTraits,
                FeaturesSubTab::SpeciesTraits => FeaturesSubTab::Feats,
                FeaturesSubTab::Feats => FeaturesSubTab::Feats,
            };
        }
        KeyCode::Char('q') => app.should_quit = true,
        // ASI/Feat choice
        KeyCode::Char('a') | KeyCode::Char('A') => {
            app.open_asi_choice();
        }
        // Subclass picker for primary class
        KeyCode::Char('s') | KeyCode::Char('S') => {
            app.open_subclass_picker(0); // 0 = primary class
        }
        KeyCode::Char('d') => {
            app.remove_selected_feat();
        }
        KeyCode::Char('w') | KeyCode::Char('W') => {
            app.picker_mode = crate::models::app_state::PickerMode::WeaponMasteryPicker;
            app.picker_search.clear();
            app.picker_selected = 0;
            // Need to fetch weapons logic later
        }
        KeyCode::Up => {
            if app.content_scroll > 0 {
                app.content_scroll -= 1;
            }
        }
        KeyCode::Down => {
            app.content_scroll += 1;
        }
        // Use feature: 'u'
        KeyCode::Char('u') | KeyCode::Char('U') => {
            app.expend_selected_feat(1);
        }
        // Recover feature: 'r'
        KeyCode::Char('r') | KeyCode::Char('R') => {
            app.expend_selected_feat(-1);
        }
        _ => {}
    }
}
