/// A strongly-typed representation of a D&D 5e feature or trait.
///
/// Converted from raw English text by `interpret_feature`.
/// The UI and builder pattern-match on this enum to decide what widget
/// to render — no raw text parsing should happen in the UI layer.
#[derive(Debug, Clone, PartialEq)]
pub enum Feature {
    /// No recognised interactive pattern — display the raw text as-is.
    StaticFeat(String),

    /// Ability Score Improvement: distribute `points` points across abilities.
    Asi { points: u8 },

    /// Choose exactly one option from a named list (e.g. "Savage Attacker or ASI").
    Choice { choose: u8, options: Vec<String> },

    /// The player must choose `choose` weapon masteries from the weapon list.
    WeaponMastery { choose: u8 },

    /// The player may choose `choose` additional skill proficiencies.
    SkillChoice { choose: u8 },

    /// This feature/feat itself grants one (or more) Origin feats.
    GrantsOriginFeat { choose: u8 },

    /// This feature grants a specific spell that is always prepared.
    GrantsSpell { spell_name: String },
}
