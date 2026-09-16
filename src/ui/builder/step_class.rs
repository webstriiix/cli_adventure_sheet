use crate::app::App;
use crate::models::{DecisionStatus, app_state::CharacterCreationStep};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

// ── Data model ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ProgressionRow {
    /// A static named feature (Source A: class feature, Source B: subclass feature).
    FeatureHeader {
        level: i32,
        name: String,
        description: String,
        /// "class" | "subclass" – used for color tinting
        source: String,
    },
    /// An interactive decision slot (Source C: manifest decision points + subclass slot).
    DecisionSlot {
        level: i32,
        /// "asi" | "weapon_mastery" | "subclass" | other
        choice_type: String,
        status: DecisionStatus,
        required_count: i32,
        current_count: i32,
        descriptions: Vec<String>,
    },
}

impl ProgressionRow {
    pub fn level(&self) -> i32 {
        match self {
            ProgressionRow::FeatureHeader { level, .. } => *level,
            ProgressionRow::DecisionSlot { level, .. } => *level,
        }
    }
}

// ── Manifest refresh ──────────────────────────────────────────────────────────

pub fn refresh_progression_manifest(app: &mut App) {
    app.refresh_progression_manifest();
}

// ── Helper: resolve subclass unlock level ────────────────────────────────────

fn subclass_unlock_level(app: &App) -> i32 {
    app.class_detail
        .as_ref()
        .and_then(|d| d.subclasses.first())
        .map(|s| s.subclass.unlock_level)
        .unwrap_or(3)
}

// ── Core build function: merged chronological roadmap (Sources A + B + C) ────
//
// ALGORITHM (called once per render frame – kept allocation-light):
//
//  For each level 1..=20:
//    1. Collect manifest decision_points at this level  (Source C – MASTER TRUTH).
//    2. Collect static class features at this level     (Source A).
//    3. Collect subclass features at this level         (Source B).
//
//    4. For every manifest decision_point:
//         a. Emit a DecisionSlot row.
//         b. Record its choice_type so we can suppress the duplicate static row.
//
//    5. For every static class feature:
//         – SKIP if the feature's name is a known ASI/WM label AND the manifest
//           already produced a slot of that type at this level.
//           We use a name-set lookup (not interpret()) to avoid false negatives
//           from the interpreter's or-choice pre-emption bug.
//         – Otherwise emit a FeatureHeader row.
//
//    6. Emit subclass feature FeatureHeader rows (no suppression needed).
//    7. Emit subclass decision slot at the class's unlock level.
//
//  SYNTHETIC FALLBACK (manifest == None):
//    When the manifest has not been fetched yet we cannot know the real slot
//    statuses.  Instead of showing nothing, we scan Source A for features
//    whose names match well-known ASI/WM labels and emit Pending DecisionSlots
//    so the tree is never empty at levels 4/8/12/16/19.
//    The same name-based suppression still runs so we never double-render.
//
//  ORDER: rows are appended in level order within the outer 1..=20 loop →
//  the final Vec is already chronological.

// ── Feature name classifiers ──────────────────────────────────────────────────
// Used only for dedup-suppression of static class features when a manifest
// slot already covers the same level+type. Never used to *create* slots.

const ASI_FEATURE_NAMES: &[&str] = &["ability score improvement", "ability score increase"];

const WM_FEATURE_NAMES: &[&str] = &["weapon mastery", "weapon masteries"];

fn feature_choice_type(name: &str) -> Option<&'static str> {
    let lower = name.to_lowercase();
    if ASI_FEATURE_NAMES.iter().any(|n| lower.contains(n)) {
        return Some("asi");
    }
    if WM_FEATURE_NAMES.iter().any(|n| lower.contains(n)) {
        return Some("weapon_mastery");
    }
    None
}

// ── Canonical ASI level fallback ──────────────────────────────────────────────
// Standard 5e 2024 ASI levels per class.  Used ONLY when the manifest has not
// been fetched yet AND the class features array does not already contain all
// the expected entries.
//
// Matching strips source-book suffixes like "[XPHB]", "[PHB]", etc. before
// comparing, so "Wizard [XPHB]" is normalised to "wizard" before the match.
fn strip_source_tag(name: &str) -> String {
    // Remove anything inside brackets at the end: "Wizard [XPHB]" → "wizard"
    let without_bracket = if let Some(idx) = name.rfind('[') {
        name[..idx].trim()
    } else {
        name.trim()
    };
    without_bracket.to_lowercase()
}

fn canonical_asi_levels(class_name: &str) -> &'static [i32] {
    let normalized = strip_source_tag(class_name);
    // Use contains() so partial matches like "eldritch knight" still hit "fighter"
    // when needed, but keep it specific enough to avoid false positives.
    if normalized.contains("fighter") {
        &[4, 6, 8, 12, 14, 16, 19]
    } else if normalized.contains("rogue") {
        &[4, 8, 10, 12, 16, 19]
    } else {
        &[4, 8, 12, 16, 19] // all other classes (Wizard, Paladin, Tamer, etc.)
    }
}

// ── Skill-choice parser ───────────────────────────────────────────────────────
// Returns (count_to_choose, allowed_skill_names) from the class skill_choices JSON.
// The JSON shape is: [{ "choose": 2, "from": ["arcana", "history", ...] }]
pub fn parse_skill_choices_pub(class: &crate::models::Class) -> (usize, Vec<String>) {
    parse_skill_choices(class)
}

fn parse_skill_choices(class: &crate::models::Class) -> (usize, Vec<String>) {
    let arr = match class.skill_choices.as_array() {
        Some(a) if !a.is_empty() => a,
        _ => return (0, Vec::new()),
    };
    let entry = &arr[0]; // classes have one skill-choice block
    let choose = entry.get("choose").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
    let from: Vec<String> = entry
        .get("from")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| {
                    v.as_str()
                        .or_else(|| v.get("name").and_then(|n| n.as_str()))
                        .map(|s| title_case(s))
                })
                .collect()
        })
        .unwrap_or_default();
    (choose, from)
}

// ── Core build function ───────────────────────────────────────────────────────
//
// PRIORITY ORDER for decision slots:
//   1. Manifest (authoritative, real server state)
//   2. Class features array (if it happens to contain ASI/WM entries)
//   3. Canonical fallback table (guarantees slots even with bad data)
//
// Static FeatureHeader rows are suppressed whenever a DecisionSlot of the
// same choice_type is being emitted for the same level, so there is never
// a dead label sitting above an interactive slot.
//
// Result is chronologically ordered because we iterate 1..=20 and append
// within each level.

pub fn build_progression_rows(app: &App, class: &crate::models::Class) -> Vec<ProgressionRow> {
    let detail_loaded = app.class_detail.as_ref().map(|d| d.class.id) == Some(class.id);

    // ── Source A: static class features ──────────────────────────────────────
    let class_features: Vec<crate::models::compendium::ClassFeature> = if detail_loaded {
        app.class_detail
            .as_ref()
            .map(|d| d.features.clone())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // ── Source B: subclass features ──────────────────────────────────────────
    let subclass_features: Vec<crate::models::compendium::SubclassFeature> = if detail_loaded {
        app.builder
            .subclass_id
            .and_then(|sc_id| {
                app.class_detail.as_ref().and_then(|d| {
                    d.subclasses
                        .iter()
                        .find(|s| s.subclass.id == sc_id)
                        .map(|s| s.features.clone())
                })
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    // ── Source C: manifest (master truth) ────────────────────────────────────
    let manifest = app.builder.progression_manifest.as_ref();

    // Pre-group manifest decision_points by level for O(1) per-level access.
    let mut manifest_by_level: std::collections::HashMap<i32, Vec<&crate::models::DecisionPoint>> =
        std::collections::HashMap::new();
    if let Some(m) = manifest {
        for dp in &m.decision_points {
            manifest_by_level.entry(dp.level).or_default().push(dp);
        }
    }

    // ── Synthetic decision slots (manifest not yet fetched) ───────────────────
    // Built once, outside the loop.  Priority:
    //   a) Scan class features for ASI/WM names (covers classes that do list them)
    //   b) Fill any still-missing ASI levels from the canonical table
    // Result: Vec<(level, choice_type)> deduplicated by (level, type).
    let synthetic_slots: Vec<(i32, &'static str)> = if manifest.is_none() {
        let mut slots: Vec<(i32, &'static str)> = Vec::new();

        // a) From class features
        for f in class_features.iter().filter(|f| !f.is_subclass_gate) {
            if let Some(ct) = feature_choice_type(&f.name) {
                if !slots.iter().any(|(l, t)| *l == f.level && *t == ct) {
                    slots.push((f.level, ct));
                }
            }
        }

        // b) Canonical ASI levels – fill gaps not covered by class features
        for &lvl in canonical_asi_levels(&class.name) {
            if !slots.iter().any(|(l, t)| *l == lvl && *t == "asi") {
                slots.push((lvl, "asi"));
            }
        }

        slots
    } else {
        Vec::new()
    };

    // ── Per-level slot coverage sets ──────────────────────────────────────────
    // Which choice_types does the current slot source cover at each level?
    // Used to suppress duplicate static FeatureHeader rows.
    // Built as a closure to keep the loop body clean.
    let covered_at = |lvl: i32| -> std::collections::HashSet<&str> {
        if manifest.is_some() {
            manifest_by_level
                .get(&lvl)
                .map(|dps| dps.iter().map(|dp| dp.choice_type.as_str()).collect())
                .unwrap_or_default()
        } else {
            synthetic_slots
                .iter()
                .filter(|(l, _)| *l == lvl)
                .map(|(_, ct)| *ct)
                .collect()
        }
    };

    let unlock_lvl = subclass_unlock_level(app);
    let mut rows: Vec<ProgressionRow> = Vec::new();

    for lvl in 1i32..=20 {
        let covered = covered_at(lvl);

        // ── Source A: static features, dedup-suppressed ───────────────────────
        for f in class_features
            .iter()
            .filter(|f| f.level == lvl && !f.is_subclass_gate)
        {
            // If a manifest/synthetic slot already covers this choice_type at
            // this level, skip the dead static label – the slot row below is
            // the canonical interactive replacement.
            if let Some(ct) = feature_choice_type(&f.name) {
                if covered.contains(ct) {
                    continue;
                }
            }
            let desc = f
                .entries
                .as_ref()
                .map(|e| crate::models::compendium::json_array_to_text(e))
                .unwrap_or_default();
            rows.push(ProgressionRow::FeatureHeader {
                level: lvl,
                name: f.name.clone(),
                description: desc,
                source: "class".to_string(),
            });
        }

        // ── Source B: subclass features ───────────────────────────────────────
        for sf in subclass_features.iter().filter(|sf| sf.level == lvl) {
            let desc = {
                let mut combined = sf.entries.clone().unwrap_or_default();
                if let Some(hdr) = &sf.header {
                    combined.insert(0, hdr.clone());
                }
                crate::models::compendium::json_array_to_text(&combined)
            };
            rows.push(ProgressionRow::FeatureHeader {
                level: lvl,
                name: sf.name.clone(),
                description: desc,
                source: "subclass".to_string(),
            });
        }

        // ── Skill proficiency slots at Level 1 ───────────────────────────────
        // Injected for every class that has skill_choices. The count and
        // allowed list come from the class JSON; choices already made are in
        // app.builder.skill_choices. Slots beyond what the character has
        // chosen show as Pending; filled ones show as Complete.
        if lvl == 1 {
            let (choose, _from) = parse_skill_choices(class);
            for slot in 0..choose {
                let chosen = app.builder.skill_choices.get(slot).cloned();
                let (status, descs, cur) = match chosen {
                    Some(skill) => (DecisionStatus::Complete, vec![skill], 1),
                    None => (DecisionStatus::Pending, vec![], 0),
                };
                rows.push(ProgressionRow::DecisionSlot {
                    level: 1,
                    choice_type: format!("skill_proficiency:{}", slot),
                    status,
                    required_count: 1,
                    current_count: cur,
                    descriptions: descs,
                });
            }
        }

        // ── Subclass decision slot ────────────────────────────────────────────
        if lvl == unlock_lvl {
            let sc_name = app.builder.subclass_id.and_then(|id| {
                app.class_detail
                    .as_ref()
                    .and_then(|d| d.subclasses.iter().find(|s| s.subclass.id == id))
                    .map(|s| s.subclass.name.clone())
            });
            let (status, descs, cur) = match sc_name {
                Some(name) => (DecisionStatus::Complete, vec![name], 1),
                None => (DecisionStatus::Pending, vec![], 0),
            };
            rows.push(ProgressionRow::DecisionSlot {
                level: lvl,
                choice_type: "subclass".to_string(),
                status,
                required_count: 1,
                current_count: cur,
                descriptions: descs,
            });
        }

        // ── Source C (authoritative): manifest decision_points ────────────────
        if let Some(dps) = manifest_by_level.get(&lvl) {
            for dp in dps.iter() {
                let (status, descs) = if dp.choice_type == "asi"
                    && (dp.status == DecisionStatus::Pending || dp.current_choices.is_empty())
                {
                    if let Some(choice) = app.builder.asi_choices.get(&lvl) {
                        (DecisionStatus::Complete, vec![choice.clone()])
                    } else {
                        (
                            dp.status,
                            dp.current_choices
                                .iter()
                                .map(|c| c.description.clone())
                                .collect(),
                        )
                    }
                } else if dp.choice_type == "weapon_mastery"
                    && (dp.status == DecisionStatus::Pending || dp.current_choices.is_empty())
                {
                    if !app.builder.weapon_mastery_choices.is_empty() {
                        (
                            DecisionStatus::Complete,
                            app.builder.weapon_mastery_choices.clone(),
                        )
                    } else {
                        (
                            dp.status,
                            dp.current_choices
                                .iter()
                                .map(|c| c.description.clone())
                                .collect(),
                        )
                    }
                } else {
                    (
                        dp.status,
                        dp.current_choices
                            .iter()
                            .map(|c| c.description.clone())
                            .collect(),
                    )
                };

                let cur_count = if status == DecisionStatus::Complete
                    && descs.len() > dp.current_choices.len()
                {
                    descs.len() as i32
                } else {
                    dp.current_choices.len() as i32
                };

                rows.push(ProgressionRow::DecisionSlot {
                    level: lvl,
                    choice_type: dp.choice_type.clone(),
                    status,
                    required_count: dp.required_count,
                    current_count: cur_count,
                    descriptions: descs,
                });
            }
        } else if manifest.is_none() {
            // ── Synthetic fallback ────────────────────────────────────────────
            for (_, ct) in synthetic_slots.iter().filter(|(l, _)| *l == lvl) {
                let (status, descs, cur) = match *ct {
                    "asi" => {
                        if let Some(choice) = app.builder.asi_choices.get(&lvl) {
                            (DecisionStatus::Complete, vec![choice.clone()], 1)
                        } else {
                            (DecisionStatus::Pending, vec![], 0)
                        }
                    }
                    "weapon_mastery" => {
                        if !app.builder.weapon_mastery_choices.is_empty() {
                            (
                                DecisionStatus::Complete,
                                app.builder.weapon_mastery_choices.clone(),
                                app.builder.weapon_mastery_choices.len() as i32,
                            )
                        } else {
                            (DecisionStatus::Pending, vec![], 0)
                        }
                    }
                    _ => (DecisionStatus::Pending, vec![], 0),
                };
                rows.push(ProgressionRow::DecisionSlot {
                    level: lvl,
                    choice_type: ct.to_string(),
                    status,
                    required_count: 1,
                    current_count: cur,
                    descriptions: descs,
                });
            }
        }
    }

    rows
}

// ── Top-level render ──────────────────────────────────────────────────────────

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    // Auto-fetch manifest when we have a server-side character/draft to query.
    // We intentionally do NOT block on a network call here if nothing is saved
    // yet – the synthetic fallback slots in build_progression_rows() will fill
    // the tree until the manifest arrives.
    if app.builder.progression_manifest.is_none()
        && (app.builder.draft_id.is_some() || app.active_character.is_some())
    {
        refresh_progression_manifest(app);
    }

    let body =
        Layout::horizontal([Constraint::Percentage(30), Constraint::Percentage(70)]).split(area);

    let focus_left = app.builder.focus_index == 0;
    let focus_right = app.builder.focus_index == 1;

    // ── Left Pane: search bar + class list ────────────────────────────────────
    let left_layout = Layout::vertical([
        Constraint::Length(3), // search bar
        Constraint::Min(0),    // class list
    ])
    .split(body[0]);

    // Search bar
    let search_border = if focus_left {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let search_text = if app.builder.class_search.is_empty() && !focus_left {
        Span::styled("Search classes…", Style::default().fg(Color::DarkGray))
    } else {
        Span::styled(
            format!("{}▌", app.builder.class_search),
            Style::default().fg(Color::White),
        )
    };
    let search_bar = Paragraph::new(Line::from(vec![search_text])).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Search ")
            .border_style(search_border),
    );
    frame.render_widget(search_bar, left_layout[0]);

    // Build filtered class list (case-insensitive substring match on name)
    let query = app.builder.class_search.to_lowercase();
    let filtered_classes: Vec<(usize, &crate::models::Class)> = app
        .classes
        .iter()
        .enumerate()
        .filter(|(_, c)| query.is_empty() || c.name.to_lowercase().contains(&query))
        .collect();

    let list_border_style = if focus_left {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = if filtered_classes.is_empty() {
        vec![ListItem::new(Span::styled(
            "  No classes found",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        filtered_classes
            .iter()
            .map(|(_, c)| {
                let is_selected = Some(c.id) == app.builder.class_id;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(c.name.clone(), style),
                    Span::styled(
                        format!(" [{}]", c.source_slug),
                        Style::default().fg(Color::DarkGray),
                    ),
                ]))
            })
            .collect()
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Class [←/h] ")
                .border_style(list_border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // Clamp list_state to filtered length
    let filtered_len = filtered_classes.len();
    if filtered_len == 0 {
        app.builder.list_state.select(None);
    } else if app
        .builder
        .list_state
        .selected()
        .map_or(true, |i| i >= filtered_len)
    {
        app.builder.list_state.select(Some(0));
    }

    frame.render_stateful_widget(list, left_layout[1], &mut app.builder.list_state);

    // ── Right Pane: Traits panel + level header + progression tree ────────────
    // Resolve which class is currently highlighted, accounting for search filter.
    let selected_class: Option<crate::models::Class> = {
        let q = app.builder.class_search.to_lowercase();
        let filtered: Vec<&crate::models::Class> = app
            .classes
            .iter()
            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
            .collect();
        app.builder
            .list_state
            .selected()
            .and_then(|i| filtered.get(i).map(|c| (*c).clone()))
    };
    let right = Layout::vertical([
        Constraint::Length(7), // Core Traits Panel
        Constraint::Length(3), // Level / subclass header
        Constraint::Min(0),    // Progression Tree
    ])
    .split(body[1]);

    // Render core traits panel using the currently highlighted class
    if let Some(ref class) = selected_class {
        render_core_traits_panel(app, frame, right[0], class);
    } else {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(" Class Traits ")
                .border_style(Style::default().fg(Color::DarkGray)),
            right[0],
        );
    }

    // Level / subclass indicator bar
    let subclass_indicator = if let Some(id) = app.builder.subclass_id {
        let sc_name = app
            .class_detail
            .as_ref()
            .and_then(|d| d.subclasses.iter().find(|s| s.subclass.id == id))
            .map(|s| s.subclass.name.clone())
            .unwrap_or_default();
        format!(" | Subclass: {}", sc_name)
    } else if app.builder.level >= subclass_unlock_level(app) {
        " | [Subclass required — press S or Enter on slot]".to_string()
    } else {
        String::new()
    };

    let level_header = Paragraph::new(Line::from(vec![
        Span::styled("Level: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{}", app.builder.level),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("  (-/+ to change)", Style::default().fg(Color::DarkGray)),
        Span::styled(subclass_indicator, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(level_header, right[1]);

    // Progression tree
    if let Some(ref class) = selected_class {
        render_progression_tree(app, frame, right[2], class, focus_right);
    }
}

// ── Core Traits Panel ─────────────────────────────────────────────────────────

fn render_core_traits_panel(
    app: &App,
    frame: &mut Frame,
    area: Rect,
    class: &crate::models::Class,
) {
    let detail_loaded = app.class_detail.as_ref().map(|d| d.class.id) == Some(class.id);

    // ── Derived data ──────────────────────────────────────────────────────────
    let die = class.hit_die;
    // D&D 2024: HP at Level 1 = max hit die + Con modifier
    let hp_lvl1 = format!("{} + Con modifier", die);
    // HP at Higher Levels: roll (or take average) + Con modifier
    // Average = floor(die/2) + 1  (e.g. d8 → 5, d10 → 6, d12 → 7)
    let hp_avg = (die / 2) + 1;
    let hp_higher = format!("1d{} (or {}) + Con modifier", die, hp_avg);

    let saves = class
        .proficiency_saves
        .as_deref()
        .map(|v| {
            v.iter()
                .map(|s| title_case(s))
                .collect::<Vec<_>>()
                .join(", ")
        })
        .unwrap_or_else(|| "—".to_string());

    let armor = class
        .armor_proficiencies
        .as_deref()
        .map(|v| v.join(", "))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "—".to_string());

    let weapons = class
        .weapon_proficiencies
        .as_deref()
        .map(|v| v.join(", "))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "—".to_string());

    let primary = class
        .spellcasting_ability
        .as_deref()
        .map(|s| title_case(s))
        .unwrap_or_else(|| "—".to_string());

    // ── Skills summary: "Choose 2 from: Arcana, History, …" ─────────────────
    // Parse the raw from-list so we can display "Choose X from: A, B, C…"
    // even before class details load (basic class data has skill_choices).
    let skills_summary: String = {
        let (choose, from) = parse_skill_choices(class);
        if choose == 0 {
            if detail_loaded {
                "None".to_string()
            } else {
                "Loading…".to_string()
            }
        } else if from.is_empty() {
            format!("Choose {}", choose)
        } else {
            format!("Choose {} from: {}", choose, from.join(", "))
        }
    };

    // ── Equipment hint ────────────────────────────────────────────────────────
    let equip_hint = if detail_loaded {
        "Choose (A) Bundle  or  (B) Gold".to_string()
    } else {
        "Loading…".to_string()
    };

    // ── Outer block ──────────────────────────────────────────────────────────
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} — Class Traits ", class.name))
        .border_style(Style::default().fg(Color::DarkGray));
    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // ── Two columns ──────────────────────────────────────────────────────────
    // Left: HP + Saves + Spellcasting + Skills
    // Right: Armor + Weapons + Equipment
    let cols =
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).split(inner);

    // Maximum usable width per column (subtract label prefix chars)
    let lw = cols[0].width.saturating_sub(13) as usize;
    let rw = cols[1].width.saturating_sub(11) as usize;

    // ── Left column ───────────────────────────────────────────────────────────
    // Line 1: HP at Level 1
    // Line 2: HP at Higher Levels
    // Line 3: Saving Throws
    // Line 4: Spellcasting ability (or blank if none)
    // Line 5: Skills summary (truncated to fit)
    let left_text = vec![
        Line::from(vec![
            Span::styled("HP  Lvl 1: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                hp_lvl1,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("HP Higher: ", Style::default().fg(Color::DarkGray)),
            Span::styled(hp_higher, Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("Saves:     ", Style::default().fg(Color::DarkGray)),
            Span::styled(saves, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Spellcast: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if primary == "—" {
                    "—".to_string()
                } else {
                    primary
                },
                Style::default().fg(Color::Magenta),
            ),
        ]),
        Line::from(vec![
            Span::styled("Skills:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                truncate_str(&skills_summary, lw),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    // ── Right column ──────────────────────────────────────────────────────────
    // Line 1: Armor training
    // Line 2: Weapon training
    // Line 3: blank spacer
    // Line 4: blank spacer
    // Line 5: Equipment hint
    let right_text = vec![
        Line::from(vec![
            Span::styled("Armor:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(truncate_str(&armor, rw), Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Weapons:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                truncate_str(&weapons, rw),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(Span::raw("")),
        Line::from(Span::raw("")),
        Line::from(vec![
            Span::styled("Equipment: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                truncate_str(&equip_hint, rw),
                Style::default().fg(Color::DarkGray),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(left_text), cols[0]);
    frame.render_widget(Paragraph::new(right_text), cols[1]);
}

// ── Progression Tree ──────────────────────────────────────────────────────────

fn render_progression_tree(
    app: &mut App,
    frame: &mut Frame,
    area: Rect,
    class: &crate::models::Class,
    is_focused: bool,
) {
    let rows = build_progression_rows(app, class);

    if rows.is_empty() {
        let loading = Paragraph::new(Span::styled(
            "Loading class details & progression…",
            Style::default().fg(Color::DarkGray),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Progression Tree (1-20) ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        frame.render_widget(loading, area);
        return;
    }

    // Clamp cursor
    if app.builder.feature_cursor >= rows.len() {
        app.builder.feature_cursor = rows.len().saturating_sub(1);
    }

    let blink = app.builder.progression_blink_tick % 2 == 0;
    let cur_level = app.builder.level;

    let tree_items: Vec<ListItem> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let is_cursor = i == app.builder.feature_cursor;
            let level = row.level();
            let is_locked = level > cur_level;
            let is_current = level == cur_level;

            match row {
                ProgressionRow::FeatureHeader { name, source, .. } => {
                    let level_str = format!("Lvl {:>2} │ ", level);
                    let (lvl_style, name_style) = if is_locked {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else if is_current {
                        (
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        let lvl_fg = if source == "subclass" {
                            Color::Magenta
                        } else {
                            Color::Cyan
                        };
                        (
                            Style::default().fg(lvl_fg),
                            Style::default().fg(Color::White),
                        )
                    };

                    // Bake the cursor indicator directly into the item so
                    // highlight_symbol on the List widget isn't needed.
                    let cursor_str = if is_cursor && is_focused {
                        ">> "
                    } else {
                        "   "
                    };
                    let cursor_style = if is_cursor && is_focused {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let badge = if source == "subclass" && !is_locked {
                        Span::styled(
                            " [SC]",
                            Style::default()
                                .fg(Color::Magenta)
                                .add_modifier(Modifier::DIM),
                        )
                    } else {
                        Span::raw("")
                    };

                    let line = Line::from(vec![
                        Span::styled(cursor_str, cursor_style),
                        Span::styled(level_str, lvl_style),
                        Span::styled(name.clone(), name_style),
                        badge,
                    ]);

                    let row_style = if is_cursor && is_focused {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    ListItem::new(line).style(row_style)
                }

                ProgressionRow::DecisionSlot {
                    choice_type,
                    status,
                    required_count,
                    current_count,
                    descriptions,
                    ..
                } => {
                    // skill_proficiency:N → slot number for display
                    let slot_num: Option<usize> = choice_type
                        .strip_prefix("skill_proficiency:")
                        .and_then(|n| n.parse().ok());

                    let label_type = if slot_num.is_some() {
                        "SKILL PROFICIENCY"
                    } else {
                        match choice_type.as_str() {
                            "asi" => "ASI / FEAT",
                            "weapon_mastery" => "WEAPON MASTERY",
                            "subclass" => "SUBCLASS",
                            other => other,
                        }
                    };

                    let level_str = format!("Lvl {:>2} │ ", level);
                    let title_name = if let Some(n) = slot_num {
                        format!("Skill Proficiency — Slot {}", n + 1)
                    } else {
                        match choice_type.as_str() {
                            "asi" => "Ability Score Improvement".to_string(),
                            "weapon_mastery" => "Weapon Mastery".to_string(),
                            "subclass" => app
                                .class_detail
                                .as_ref()
                                .and_then(|d| d.class.subclass_title.clone())
                                .unwrap_or_else(|| "Subclass".to_string()),
                            other => title_case(other),
                        }
                    };

                    let (title_lvl_style, title_name_style) = if is_locked {
                        (
                            Style::default().fg(Color::DarkGray),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else if is_current {
                        (
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        (
                            Style::default().fg(Color::Cyan),
                            Style::default().fg(Color::White),
                        )
                    };

                    // Same baked cursor indicator on the title line
                    let cursor_str = if is_cursor && is_focused {
                        ">> "
                    } else {
                        "   "
                    };
                    let cursor_style = if is_cursor && is_focused {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    };

                    let title_line = Line::from(vec![
                        Span::styled(cursor_str, cursor_style),
                        Span::styled(level_str, title_lvl_style),
                        Span::styled(title_name, title_name_style),
                    ]);

                    // ── Choice sub-row ────────────────────────────────────────
                    // Indented by the same width as the cursor prefix ("   ")
                    // so the ╰─ sits neatly under the level label.
                    let slot_span = if is_locked {
                        Span::styled(
                            "     ╰─ [ LOCKED ]".to_string(),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else {
                        match status {
                            DecisionStatus::Pending => {
                                let fg = if blink {
                                    Color::Yellow
                                } else {
                                    Color::DarkGray
                                };
                                Span::styled(
                                    format!("     ╰─ [ SELECT {} ]", label_type),
                                    Style::default().fg(fg).add_modifier(Modifier::BOLD),
                                )
                            }
                            DecisionStatus::Partial => Span::styled(
                                format!(
                                    "     ╰─ [ SELECT MORE ({}/{}) ]",
                                    current_count, required_count
                                ),
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            DecisionStatus::Complete => {
                                let desc_str = if descriptions.is_empty() {
                                    "Complete".to_string()
                                } else {
                                    descriptions.join(", ")
                                };
                                Span::styled(
                                    format!("     ╰─ [ {} ]", desc_str),
                                    Style::default().fg(Color::Green),
                                )
                            }
                        }
                    };

                    let slot_line = Line::from(vec![slot_span]);

                    let row_style = if is_cursor && is_focused {
                        Style::default().bg(Color::DarkGray)
                    } else {
                        Style::default()
                    };
                    ListItem::new(vec![title_line, slot_line]).style(row_style)
                }
            }
        })
        .collect();

    let border_style = if is_focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    app.builder
        .feature_list_state
        .select(Some(app.builder.feature_cursor));

    let tree_list = List::new(tree_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(
                    " 1-20 Progression Tree [→/l focus | J/K navigate | Enter select | Ctrl+K detail] ",
                )
                .border_style(border_style),
        )
        .highlight_style(Style::default().bg(Color::DarkGray));
    // highlight_symbol intentionally omitted — the ">>" cursor marker is baked
    // directly into each ListItem so 2-line DecisionSlot items don't cause
    // visual jumping when navigating.

    frame.render_stateful_widget(tree_list, area, &mut app.builder.feature_list_state);
}

// ── Decision helpers ──────────────────────────────────────────────────────────

pub fn has_unfulfilled_choices(app: &App) -> bool {
    let q = app.builder.class_search.to_lowercase();
    let filtered: Vec<&crate::models::Class> = app
        .classes
        .iter()
        .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
        .collect();
    let maybe_class = app
        .builder
        .list_state
        .selected()
        .and_then(|i| filtered.get(i).map(|c| (*c).clone()));
    if let Some(class) = maybe_class {
        let rows = build_progression_rows(app, &class);
        for r in &rows {
            if r.level() <= app.builder.level {
                if let ProgressionRow::DecisionSlot {
                    status,
                    choice_type,
                    ..
                } = r
                {
                    // Skill proficiency slots are always at level 1; always check them.
                    let is_skill = choice_type.starts_with("skill_proficiency:");
                    if is_skill
                        || *status == DecisionStatus::Pending
                        || *status == DecisionStatus::Partial
                    {
                        if *status != DecisionStatus::Complete {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

// ── Key handler ───────────────────────────────────────────────────────────────

pub fn handle_key(app: &mut App, key: KeyEvent) {
    // Tick blink counter on every keypress
    app.builder.progression_blink_tick = app.builder.progression_blink_tick.wrapping_add(1);

    let rows_count = {
        let q = app.builder.class_search.to_lowercase();
        let filtered: Vec<&crate::models::Class> = app
            .classes
            .iter()
            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
            .collect();
        app.builder
            .list_state
            .selected()
            .and_then(|i| {
                filtered
                    .get(i)
                    .map(|c| build_progression_rows(app, c).len())
            })
            .unwrap_or(0)
    };

    // ── Ctrl+K: feature detail modal ──────────────────────────────────────────
    if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL) {
        let q = app.builder.class_search.to_lowercase();
        let filtered: Vec<&crate::models::Class> = app
            .classes
            .iter()
            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
            .collect();
        let maybe_class = app
            .builder
            .list_state
            .selected()
            .and_then(|i| filtered.get(i).map(|c| (*c).clone()));
        if let Some(class) = maybe_class {
            let rows = build_progression_rows(app, &class);
            if let Some(row) = rows.get(app.builder.feature_cursor) {
                match row {
                    ProgressionRow::FeatureHeader {
                        name,
                        description,
                        level,
                        source,
                    } => {
                        let src_label = if source == "subclass" {
                            " (Subclass Feature)"
                        } else {
                            ""
                        };
                        let title = format!("Level {} Feature{}: {}", level, src_label, name);
                        app.builder.feature_detail_modal = Some((title, description.clone()));
                    }
                    ProgressionRow::DecisionSlot {
                        level,
                        choice_type,
                        descriptions,
                        status,
                        ..
                    } => {
                        let title = format!("Level {} Slot: {}", level, choice_type.to_uppercase());
                        let body = if descriptions.is_empty() {
                            format!(
                                "Status: {:?}\n\nNo choices recorded for this slot yet.\nPress Enter to make a selection.",
                                status
                            )
                        } else {
                            format!(
                                "Status: {:?}\n\nCurrent Selections:\n• {}",
                                status,
                                descriptions.join("\n• ")
                            )
                        };
                        app.builder.feature_detail_modal = Some((title, body));
                    }
                }
            }
        }
        return;
    }

    // ── Normal key dispatch ───────────────────────────────────────────────────
    match key.code {
        KeyCode::Esc => {
            app.screen = crate::models::app_state::Screen::CharacterList;
            app.builder = crate::models::app_state::BuilderState::default();
        }

        // Pane focus
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
            app.builder.focus_index = 0;
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
            app.builder.focus_index = 1;
        }

        // Navigation
        KeyCode::Up => {
            if app.builder.focus_index == 0 {
                let q = app.builder.class_search.to_lowercase();
                let filtered_len = app
                    .classes
                    .iter()
                    .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
                    .count();
                if filtered_len > 0 {
                    let i = match app.builder.list_state.selected() {
                        Some(0) | None => filtered_len - 1,
                        Some(i) => i - 1,
                    };
                    app.builder.list_state.select(Some(i));
                    load_class_detail_for_filtered(app);
                }
            } else {
                app.builder.feature_cursor = app.builder.feature_cursor.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if app.builder.focus_index == 0 {
                let q = app.builder.class_search.to_lowercase();
                let filtered_len = app
                    .classes
                    .iter()
                    .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
                    .count();
                if filtered_len > 0 {
                    let i = match app.builder.list_state.selected() {
                        Some(i) if i + 1 < filtered_len => i + 1,
                        _ => 0,
                    };
                    app.builder.list_state.select(Some(i));
                    load_class_detail_for_filtered(app);
                }
            } else if rows_count > 0 {
                app.builder.feature_cursor = (app.builder.feature_cursor + 1).min(rows_count - 1);
            }
        }

        // Vim-style navigation always targets the right pane
        KeyCode::Char('k') | KeyCode::Char('K') => {
            app.builder.focus_index = 1;
            app.builder.feature_cursor = app.builder.feature_cursor.saturating_sub(1);
        }
        KeyCode::Char('j') | KeyCode::Char('J') => {
            app.builder.focus_index = 1;
            if rows_count > 0 {
                app.builder.feature_cursor = (app.builder.feature_cursor + 1).min(rows_count - 1);
            }
        }

        // Level adjustment
        KeyCode::Char('-') | KeyCode::Char('_') => {
            if app.builder.level > 1 {
                app.builder.level -= 1;
                // Clear subclass if we drop below its unlock level
                if app.builder.level < subclass_unlock_level(app) {
                    app.builder.subclass_id = None;
                }
                // Persist the new level so the backend can return the correct
                // manifest (decision_points are filtered by total_level).
                if app.builder.class_id.is_some() && app.builder.draft_id.is_some() {
                    let _ = app.save_draft();
                    refresh_progression_manifest(app);
                }
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.builder.level < 20 {
                app.builder.level += 1;
            }
            // Persist the new level so the backend can return the correct
            // manifest (decision_points are filtered by total_level).
            if app.builder.class_id.is_some() && app.builder.draft_id.is_some() {
                let _ = app.save_draft();
                refresh_progression_manifest(app);
            }
            // Prompt for subclass if we just reached the unlock level
            let unlock = subclass_unlock_level(app);
            if app.builder.level >= unlock && app.builder.subclass_id.is_none() {
                let has_sc = app
                    .class_detail
                    .as_ref()
                    .map(|d| !d.subclasses.is_empty())
                    .unwrap_or(false);
                if has_sc {
                    app.builder.show_subclass_modal = true;
                    app.builder.subclass_list_state.select(Some(0));
                }
            }
        }

        // Manual subclass picker shortcut
        KeyCode::Char('s') | KeyCode::Char('S') => {
            let unlock = subclass_unlock_level(app);
            if app.builder.level >= unlock {
                let has_sc = app
                    .class_detail
                    .as_ref()
                    .map(|d| !d.subclasses.is_empty())
                    .unwrap_or(false);
                if has_sc {
                    app.builder.show_subclass_modal = true;
                    app.builder.subclass_list_state.select(Some(0));
                } else {
                    app.status_msg = "No subclasses available for this class.".to_string();
                }
            } else {
                app.status_msg = format!(
                    "Reach level {} to pick a subclass (currently level {}).",
                    unlock, app.builder.level
                );
            }
        }

        // Tab: proceed to next step
        KeyCode::Tab => {
            if has_unfulfilled_choices(app) {
                app.status_msg = "Please complete all choices for your current level.".to_string();
                return;
            }
            let q = app.builder.class_search.to_lowercase();
            let filtered: Vec<&crate::models::Class> = app
                .classes
                .iter()
                .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
                .collect();
            let maybe_class = app
                .builder
                .list_state
                .selected()
                .and_then(|i| filtered.get(i).map(|c| (*c).clone()));
            if let Some(class) = maybe_class {
                app.builder.class_id = Some(class.id);
                if !app.save_draft() {
                    return;
                }
                refresh_progression_manifest(app);
                app.builder.step = CharacterCreationStep::Background;
                app.builder.list_state.select(Some(0));
                app.builder.class_search.clear();
                app.status_msg.clear();
            }
        }

        // Enter: confirm class (left pane) or activate slot (right pane)
        KeyCode::Enter => {
            if app.builder.focus_index == 0 {
                // Confirm the highlighted class from the *filtered* list
                let q = app.builder.class_search.to_lowercase();
                let filtered: Vec<(usize, &crate::models::Class)> = app
                    .classes
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| q.is_empty() || c.name.to_lowercase().contains(&q))
                    .collect();
                if let Some(filter_idx) = app.builder.list_state.selected() {
                    if let Some((_, class)) = filtered.get(filter_idx) {
                        let class_id = class.id;
                        let caster = class.caster_progression.clone();

                        if app
                            .class_detail
                            .as_ref()
                            .map(|d| d.class.id != class_id)
                            .unwrap_or(true)
                        {
                            load_class_detail_for_filtered(app);
                        }

                        app.builder.class_id = Some(class_id);
                        app.builder.spellcasting_type = caster
                            .filter(|p| !p.is_empty() && p != "none")
                            .unwrap_or_else(|| "none".to_string());

                        if app.save_draft() {
                            refresh_progression_manifest(app);
                        }

                        let unlock = subclass_unlock_level(app);
                        if app.builder.level >= unlock && app.builder.subclass_id.is_none() {
                            let has_sc = app
                                .class_detail
                                .as_ref()
                                .map(|d| !d.subclasses.is_empty())
                                .unwrap_or(false);
                            if has_sc {
                                app.builder.show_subclass_modal = true;
                                app.builder.subclass_list_state.select(Some(0));
                                return;
                            }
                        }

                        app.builder.focus_index = 1;
                    }
                }
            } else {
                // Activate focused slot in progression tree (use filtered class lookup)
                let q = app.builder.class_search.to_lowercase();
                let filtered: Vec<&crate::models::Class> = app
                    .classes
                    .iter()
                    .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
                    .collect();
                let maybe_class = app
                    .builder
                    .list_state
                    .selected()
                    .and_then(|i| filtered.get(i).map(|c| (*c).clone()));

                if let Some(class) = maybe_class {
                    let rows = build_progression_rows(app, &class);
                    if let Some(row) = rows.get(app.builder.feature_cursor) {
                        if row.level() > app.builder.level {
                            app.status_msg = format!(
                                "Level {} is locked (currently level {}). Use '+' to level up.",
                                row.level(),
                                app.builder.level
                            );
                            return;
                        }

                        match row {
                            ProgressionRow::FeatureHeader {
                                name, description, ..
                            } => {
                                app.builder.feature_detail_modal =
                                    Some((name.clone(), description.clone()));
                            }
                            ProgressionRow::DecisionSlot {
                                level, choice_type, ..
                            } => {
                                // skill_proficiency:N → open the skill picker modal
                                if let Some(slot_str) =
                                    choice_type.strip_prefix("skill_proficiency:")
                                {
                                    if let Ok(slot_idx) = slot_str.parse::<usize>() {
                                        app.builder.show_skill_choice_modal = true;
                                        app.builder.skill_choice_slot = slot_idx;
                                        app.builder.skill_choice_search.clear();
                                        app.builder.skill_choice_cursor = 0;
                                        app.builder.skill_choice_list_state.select(Some(0));
                                    }
                                    return;
                                }
                                match choice_type.as_str() {
                                    "asi" => {
                                        app.builder.show_progression_asi_modal = true;
                                        app.builder.asi_modal_stage =
                                            crate::models::app_state::AsiModalStage::Mode;
                                        app.builder.progression_slot_level = Some(*level);
                                    }
                                    "weapon_mastery" => {
                                        app.builder.show_progression_wm_modal = true;
                                        app.builder.progression_slot_level = Some(*level);
                                    }
                                    "subclass" => {
                                        let has_sc = app
                                            .class_detail
                                            .as_ref()
                                            .map(|d| !d.subclasses.is_empty())
                                            .unwrap_or(false);
                                        if has_sc {
                                            app.builder.show_subclass_modal = true;
                                            app.builder.subclass_list_state.select(Some(0));
                                        } else {
                                            app.status_msg =
                                                "No subclasses available for this class."
                                                    .to_string();
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        // Search: Backspace removes last char when left pane focused
        KeyCode::Backspace if app.builder.focus_index == 0 => {
            app.builder.class_search.pop();
            // Reset to top of filtered list; do NOT load class detail here —
            // wait for the user to move the cursor or press Enter.
            app.builder.list_state.select(Some(0));
        }

        // Search: printable chars type into the search bar when left pane focused.
        // We deliberately do NOT call load_class_detail_for_filtered here —
        // every keystroke would fire a blocking network request causing visible lag.
        // Detail is loaded only when the selection moves (Up/Down) or on Enter.
        KeyCode::Char(c) if app.builder.focus_index == 0 => {
            if !key.modifiers.contains(KeyModifiers::CONTROL)
                && !key.modifiers.contains(KeyModifiers::ALT)
            {
                app.builder.class_search.push(c);
                app.builder.list_state.select(Some(0));
                // No network call here — intentional.
            }
        }

        _ => {}
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Load class detail for whichever class is currently highlighted in the
/// *filtered* list.  All navigation handlers call this instead of the old
/// raw-index version so that search filtering never produces a mismatched
/// class_detail.
fn load_class_detail_for_filtered(app: &mut App) {
    load_class_detail_for_current(app);
}

/// Public version of the above — called from other steps when navigating
/// back to the Class step so the correct class detail is loaded immediately.
pub fn load_class_detail_for_current(app: &mut App) {
    let q = app.builder.class_search.to_lowercase();
    // Collect target data first to avoid holding a shared borrow on app.classes
    // while we mutably borrow the rest of app below.
    let target: Option<(i32, String, String)> = {
        let filtered: Vec<&crate::models::Class> = app
            .classes
            .iter()
            .filter(|c| q.is_empty() || c.name.to_lowercase().contains(&q))
            .collect();
        app.builder
            .list_state
            .selected()
            .and_then(|i| filtered.get(i))
            .map(|c| (c.id, c.name.clone(), c.source_slug.clone()))
    };

    if let Some((id, name, source)) = target {
        let current_id = app.class_detail.as_ref().map(|d| d.class.id);
        if current_id != Some(id) {
            let rt = app.rt.clone();
            let client = app.client.clone();
            if let Ok(detail) = rt.block_on(client.get_class_detail(&name, &source)) {
                app.class_detail = Some(detail);
            }
        }
    }
}

/// Convert an ASCII identifier like "intelligence" → "Intelligence".
fn title_case(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

/// Truncate a string to `max_chars`, appending "…" if it was cut.
fn truncate_str(s: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let cut: String = chars[..max_chars.saturating_sub(1)].iter().collect();
        format!("{}…", cut)
    }
}

// ── Unit Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_skill_choices_returns_count_and_options() {
        let class: crate::models::Class = serde_json::from_value(json!({
            "id": 1,
            "name": "Rogue",
            "source_slug": "phb",
            "hit_die": 8,
            "skill_choices": [{"choose": 2, "from": ["acrobatics", "history", "insight"]}],
            "starting_equipment": null
        }))
        .unwrap();
        let (count, options) = parse_skill_choices_pub(&class);
        assert_eq!(count, 2);
        assert_eq!(options, vec!["Acrobatics", "History", "Insight"]); // title_case diterapkan
    }

    #[test]
    fn test_parse_skill_choices_empty() {
        let class: crate::models::Class = serde_json::from_value(json!({
            "id": 2,
            "name": "Test",
            "source_slug": "phb",
            "hit_die": 8,
            "skill_choices": [],
            "starting_equipment": null
        }))
        .unwrap();
        let (count, options) = parse_skill_choices_pub(&class);
        assert_eq!(count, 0);
        assert!(options.is_empty());
    }

    #[test]
    fn test_parse_skill_choices_from_object_items() {
        let class: crate::models::Class = serde_json::from_value(json!({
            "id": 3,
            "name": "Fighter",
            "source_slug": "phb",
            "hit_die": 10,
            "skill_choices": [{"choose": 1, "from": [{"name": "athletics"}, "perception"]}],
            "starting_equipment": null
        }))
        .unwrap();
        let (count, options) = parse_skill_choices_pub(&class);
        assert_eq!(count, 1);
        assert_eq!(options, vec!["Athletics", "Perception"]);
    }
}
