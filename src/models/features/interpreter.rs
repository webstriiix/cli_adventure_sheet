use super::feature::Feature;

/// Convert a raw English feature/trait description into a structured [`Feature`].
///
/// Uses lowercase keyword heuristics — no heavy NLP, no panics.
/// Always returns *something*; unknown text falls back to [`Feature::StaticFeat`].
///
/// # Examples
/// ```
/// use crate::models::features::interpreter::interpret_feature;
/// use crate::models::features::feature::Feature;
/// assert_eq!(interpret_feature("Choose 2 weapon masteries"), Feature::WeaponMastery { choose: 2 });
/// assert_eq!(interpret_feature("Ability Score Improvement"), Feature::Asi { points: 2 });
/// ```
pub fn interpret_feature(text: &str) -> Feature {
    let lower = text.to_lowercase();

    // ── Origin / bonus feat grants ──────────────────────────────────────────
    // e.g. "You gain one Origin feat", "choose a bonus feat"
    if lower.contains("origin feat") || lower.contains("bonus feat") {
        let n = extract_number(&lower).unwrap_or(1);
        return Feature::GrantsOriginFeat { choose: n };
    }

    // ── Grants Spell (2024 always prepared) ──────────────────────────────────
    // e.g. "You always have the Divine Smite spell prepared."
    if lower.contains("always have the") && lower.contains("spell prepared") {
        // Try to extract the name between "always have the " and " spell prepared"
        if let Some(start) = lower.find("always have the ") {
            if let Some(end) = lower[start + 16..].find(" spell prepared") {
                let name = text[start + 16..start + 16 + end].trim();
                if !name.is_empty() {
                    return Feature::GrantsSpell {
                        spell_name: name.to_string(),
                    };
                }
            }
        }
    }

    // ── Weapon Mastery ──────────────────────────────────────────────────────
    // e.g. "Choose 2 weapon masteries", "you learn one Weapon Mastery"
    if lower.contains("weapon master") {
        let n = extract_number(&lower).unwrap_or(1);
        return Feature::WeaponMastery { choose: n };
    }

    // ── Binary / named choice ────────────────────────────────────────────────
    // Check BEFORE keyword rules so "Savage Attacker or Ability Score Improvement"
    // resolves as a Choice rather than being absorbed by the ASI check.
    // Only triggers when " or " appears exactly once.
    if let Some(opts) = try_parse_or_choice(&lower, text) {
        return Feature::Choice {
            choose: 1,
            options: opts,
        };
    }

    // ── Ability Score Improvement ────────────────────────────────────────────
    // e.g. "Ability Score Improvement", "increase one Ability Score by 2"
    if lower.contains("ability score improvement")
        || lower.contains("increase one ability score")
        || lower.contains("increase two ability scores")
        || lower.contains("ability score increase")
    {
        return Feature::Asi { points: 2 };
    }

    // ── Skill choice ────────────────────────────────────────────────────────
    // e.g. "Choose any 3 skills", "you gain proficiency in one skill of your choice"
    if (lower.contains("skill") && lower.contains("proficiency") && lower.contains("choice"))
        || (lower.contains("choose") && lower.contains("skill"))
        || (lower.contains("gain proficiency in") && lower.contains("skill"))
    {
        let n = extract_number(&lower).unwrap_or(1);
        return Feature::SkillChoice { choose: n };
    }

    // ── Fallback ─────────────────────────────────────────────────────────────
    Feature::StaticFeat(text.to_string())
}

// ── Internal helpers ─────────────────────────────────────────────────────────

/// Pull the first standalone integer out of `text` (e.g. "Choose 2 weapon" → 2).
fn extract_number(text: &str) -> Option<u8> {
    // Walk the words; return the first one that parses as a digit.
    for word in text.split_whitespace() {
        // Strip surrounding punctuation before trying to parse.
        let clean: String = word.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(n) = clean.parse::<u8>() {
            if n > 0 {
                return Some(n);
            }
        }
        // Also handle spelled-out English words for 1–5.
        match word.trim_matches(|c: char| !c.is_alphabetic()) {
            "one" => return Some(1),
            "two" => return Some(2),
            "three" => return Some(3),
            "four" => return Some(4),
            "five" => return Some(5),
            _ => {}
        }
    }
    None
}

/// If the text looks like "Option A or Option B", return `Some(["Option A", "Option B"])`.
/// We only trigger this when the phrase contains " or " and neither side contains another " or ".
fn try_parse_or_choice(lower: &str, original: &str) -> Option<Vec<String>> {
    // Only trigger when " or " appears exactly once to keep it unambiguous.
    let or_count = lower.matches(" or ").count();
    if or_count != 1 {
        return None;
    }

    // Use the original (not lowercased) text to preserve proper nouns.
    // Find the " or " in a case-insensitive way.
    let idx = find_case_insensitive(original, " or ")?;
    let left = original[..idx].trim().to_string();
    let right = original[idx + 4..].trim().to_string();

    // Reject if either side is empty or suspiciously long (likely a sentence, not a choice).
    if left.is_empty() || right.is_empty() || left.len() > 60 || right.len() > 60 {
        return None;
    }

    Some(vec![left, right])
}

/// Find the byte index of `needle` in `haystack` (case-insensitive, first occurrence).
fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let h = haystack.to_lowercase();
    let n = needle.to_lowercase();
    h.find(&n)
}

// ── Unit tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_mastery_numeric() {
        assert_eq!(
            interpret_feature("Choose 2 weapon masteries"),
            Feature::WeaponMastery { choose: 2 }
        );
    }

    #[test]
    fn test_weapon_mastery_one() {
        assert_eq!(
            interpret_feature("You learn one Weapon Mastery of your choice."),
            Feature::WeaponMastery { choose: 1 }
        );
    }

    #[test]
    fn test_asi_standard() {
        assert_eq!(
            interpret_feature("Ability Score Improvement"),
            Feature::Asi { points: 2 }
        );
    }

    #[test]
    fn test_asi_variant() {
        assert_eq!(
            interpret_feature("Increase one Ability Score of your choice by 2."),
            Feature::Asi { points: 2 }
        );
    }

    #[test]
    fn test_origin_feat_worded() {
        assert_eq!(
            interpret_feature("You gain one Origin feat of your choice."),
            Feature::GrantsOriginFeat { choose: 1 }
        );
    }

    #[test]
    fn test_bonus_feat() {
        assert_eq!(
            interpret_feature("Choose a bonus feat."),
            Feature::GrantsOriginFeat { choose: 1 }
        );
    }

    #[test]
    fn test_skill_choice_generic() {
        assert_eq!(
            interpret_feature("Choose any 3 skills from the list below."),
            Feature::SkillChoice { choose: 3 }
        );
    }

    #[test]
    fn test_skill_proficiency_choice() {
        assert_eq!(
            interpret_feature("You gain proficiency in one skill of your choice."),
            Feature::SkillChoice { choose: 1 }
        );
    }

    #[test]
    fn test_or_choice() {
        let result = interpret_feature("Savage Attacker or Ability Score Improvement");
        assert_eq!(
            result,
            Feature::Choice {
                choose: 1,
                options: vec![
                    "Savage Attacker".to_string(),
                    "Ability Score Improvement".to_string()
                ]
            }
        );
    }

    #[test]
    fn test_static_fallback() {
        let text = "You have darkvision out to a range of 60 feet.";
        assert!(matches!(
            interpret_feature(text),
            Feature::StaticFeat(s) if s == text
        ));
    }

    #[test]
    fn test_empty_string() {
        assert!(matches!(interpret_feature(""), Feature::StaticFeat(_)));
    }
}
