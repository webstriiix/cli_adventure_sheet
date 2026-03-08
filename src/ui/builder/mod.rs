use crossterm::event::KeyEvent;
use ratatui::Frame;

use crate::app::App;
use crate::models::app_state::CharacterCreationStep;

pub mod step_abilities;
pub mod step_background;
pub mod step_bg_abilities;
pub mod step_background_feat;
pub mod step_class;
pub mod step_details;
pub mod step_equipment;
pub mod step_feat_skill;
pub mod step_languages;
pub mod step_proficiencies;
pub mod step_race;
pub mod step_race_feat;
pub mod step_race_skill;
pub mod step_spells;
pub mod step_subclass;
pub mod step_summary;
pub mod step_weapon_mastery;

pub fn render(app: &mut App, frame: &mut Frame) {
    match app.builder.step {
        CharacterCreationStep::Race => step_race::render(app, frame),
        CharacterCreationStep::RaceSkill => step_race_skill::render(app, frame),
        CharacterCreationStep::RaceFeat => step_race_feat::render(app, frame),
        CharacterCreationStep::Class => step_class::render(app, frame),
        CharacterCreationStep::Subclass => step_subclass::render(app, frame),
        CharacterCreationStep::Abilities => step_abilities::render(app, frame),
        CharacterCreationStep::Background => step_background::render(app, frame),
        CharacterCreationStep::BackgroundAbilities => step_bg_abilities::render(app, frame),
        CharacterCreationStep::Languages => step_languages::render(app, frame),
        CharacterCreationStep::Proficiencies => step_proficiencies::render(app, frame),
        CharacterCreationStep::Equipment => step_equipment::render(app, frame),
        CharacterCreationStep::Spells => step_spells::render(app, frame),
        CharacterCreationStep::Details => step_details::render(app, frame),
        CharacterCreationStep::Summary => step_summary::render(app, frame),
        CharacterCreationStep::BackgroundFeat => step_background_feat::render(app, frame),
        CharacterCreationStep::FeatWeaponMastery => step_weapon_mastery::render(app, frame),
        CharacterCreationStep::FeatSkillChoice => step_feat_skill::render(app, frame),
    }
}

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match app.builder.step {
        CharacterCreationStep::Race => step_race::handle_key(app, key),
        CharacterCreationStep::RaceSkill => step_race_skill::handle_key(app, key),
        CharacterCreationStep::RaceFeat => step_race_feat::handle_key(app, key),
        CharacterCreationStep::Class => step_class::handle_key(app, key),
        CharacterCreationStep::Subclass => step_subclass::handle_key(app, key),
        CharacterCreationStep::Abilities => step_abilities::handle_key(app, key),
        CharacterCreationStep::Background => step_background::handle_key(app, key),
        CharacterCreationStep::BackgroundAbilities => step_bg_abilities::handle_key(app, key),
        CharacterCreationStep::Languages => step_languages::handle_key(app, key),
        CharacterCreationStep::Proficiencies => step_proficiencies::handle_key(app, key),
        CharacterCreationStep::Equipment => step_equipment::handle_key(app, key),
        CharacterCreationStep::Spells => step_spells::handle_key(app, key),
        CharacterCreationStep::Details => step_details::handle_key(app, key),
        CharacterCreationStep::Summary => step_summary::handle_key(app, key),
        CharacterCreationStep::BackgroundFeat => step_background_feat::handle_key(app, key),
        CharacterCreationStep::FeatWeaponMastery => step_weapon_mastery::handle_key(app, key),
        CharacterCreationStep::FeatSkillChoice => step_feat_skill::handle_key(app, key),
    }
}
