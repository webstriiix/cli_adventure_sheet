use crate::app::App;
use crate::models::{app_state::CharacterCreationStep, DecisionStatus};
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
    let char_id = app
        .builder
        .draft_id
        .or_else(|| app.active_character.as_ref().map(|c| c.id));

    if let Some(id) = char_id {
        let rt = app.rt.clone();
        let client = app.client.clone();
        // Always overwrite: the manifest must reflect the *current* class+level
        // combination, not a stale snapshot from a prior class selection.
        match rt.block_on(client.get_progression_manifest(id)) {
            Ok(manifest) => {
                app.builder.progression_manifest = Some(manifest);
            }
            Err(e) => {
                // Don't clobber a good existing manifest on a transient failure;
                // just log and let the tree continue with synthetic slots.
                tracing::warn!("Could not fetch progression manifest: {}", e);
            }
        }
    }
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

/// Static lowercase names that indicate an ASI feature in the class feature list.
/// We match by name (not interpret()) to avoid false negatives from the interpreter.
const ASI_FEATURE_NAMES: &[&str] = &[
    "ability score improvement",
    "ability score increase",
    "feat",                     // Some classes list it as just "Feat"
];

/// Static lowercase names that indicate a Weapon Mastery feature.
const WM_FEATURE_NAMES: &[&str] = &[
    "weapon mastery",
    "weapon masteries",
];

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

    // ── Source C: manifest ────────────────────────────────────────────────────
    let manifest = app.builder.progression_manifest.as_ref();

    // Pre-index manifest decision_points by level for O(1) lookup.
    // Vec is small (max ~10 entries) so a nested scan is fine, but let's
    // group by level to make the per-level logic clean.
    let mut manifest_by_level: std::collections::HashMap<i32, Vec<&crate::models::DecisionPoint>> =
        std::collections::HashMap::new();
    if let Some(m) = manifest {
        for dp in &m.decision_points {
            manifest_by_level.entry(dp.level).or_default().push(dp);
        }
    }

    // ── Synthetic fallback index (manifest not yet fetched) ───────────────────
    // Scan Source A once for features whose names look like ASI or WM slots.
    // Stored as (level, choice_type).
    let synthetic_slots: Vec<(i32, &'static str)> = if manifest.is_none() {
        class_features
            .iter()
            .filter(|f| !f.is_subclass_gate)
            .filter_map(|f| {
                feature_choice_type(&f.name).map(|ct| (f.level, ct))
            })
            .collect()
    } else {
        Vec::new()
    };

    let unlock_lvl = subclass_unlock_level(app);
    let mut rows: Vec<ProgressionRow> = Vec::new();

    for lvl in 1i32..=20 {
        // ── Step 1: Determine which choice_types are covered at this level ────
        // This drives the suppression of duplicate static feature rows.
        let covered_types: std::collections::HashSet<&str> = if manifest.is_some() {
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
        };

        // ── Step 2: Source A – static class features with dedup suppression ───
        for f in class_features
            .iter()
            .filter(|f| f.level == lvl && !f.is_subclass_gate)
        {
            // If the manifest (or synthetic fallback) already covers this
            // feature's slot type, skip the static header – the DecisionSlot
            // row emitted below is the canonical, interactive replacement.
            if let Some(ct) = feature_choice_type(&f.name) {
                if covered_types.contains(ct) {
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

        // ── Step 3: Source B – subclass features ──────────────────────────────
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

        // ── Step 4: Subclass decision slot ────────────────────────────────────
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

        // ── Step 5a: Source C (authoritative) – manifest decision_points ──────
        if let Some(dps) = manifest_by_level.get(&lvl) {
            for dp in dps.iter() {
                let descs: Vec<String> = dp
                    .current_choices
                    .iter()
                    .map(|c| c.description.clone())
                    .collect();
                rows.push(ProgressionRow::DecisionSlot {
                    level: lvl,
                    choice_type: dp.choice_type.clone(),
                    status: dp.status,
                    required_count: dp.required_count,
                    current_count: dp.current_choices.len() as i32,
                    descriptions: descs,
                });
            }
        } else if manifest.is_none() {
            // ── Step 5b: Synthetic fallback – no manifest yet ─────────────────
            for (_, ct) in synthetic_slots.iter().filter(|(l, _)| *l == lvl) {
                rows.push(ProgressionRow::DecisionSlot {
                    level: lvl,
                    choice_type: ct.to_string(),
                    status: DecisionStatus::Pending,
                    required_count: 1,
                    current_count: 0,
                    descriptions: vec![],
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

    // ── Left Pane: Class list ──────────────────────────────────────────────────
    let items: Vec<ListItem> = app
        .classes
        .iter()
        .map(|c| {
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
        .collect();

    let list_border_style = if focus_left {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Select Class [Focus: ←/h] ")
                .border_style(list_border_style),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, body[0], &mut app.builder.list_state);

    // ── Right Pane: Traits panel + level header + progression tree ────────────
    let right = Layout::vertical([
        Constraint::Length(5), // Core Traits Panel
        Constraint::Length(3), // Level / subclass header
        Constraint::Min(0),    // Progression Tree
    ])
    .split(body[1]);

    // Render core traits panel using the currently highlighted class
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx).cloned() {
            render_core_traits_panel(app, frame, right[0], &class);
        }
    } else {
        // Empty placeholder so the border still shows
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
        Span::styled(
            "  (-/+ to change)",
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(subclass_indicator, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(level_header, right[1]);

    // Progression tree
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx).cloned() {
            render_progression_tree(app, frame, right[2], &class, focus_right);
        }
    }
}

// ── Core Traits Panel ─────────────────────────────────────────────────────────

fn render_core_traits_panel(
    app: &App,
    frame: &mut Frame,
    area: Rect,
    class: &crate::models::Class,
) {
    // Retrieve extra class detail only if it's already loaded; we never block here
    let detail_loaded = app.class_detail.as_ref().map(|d| d.class.id) == Some(class.id);
    let tool_profs: String = if detail_loaded {
        // Class model doesn't carry tool_proficiencies as a field, so we parse from features
        // (a common source is "Thieves' Tools" in Rogue features, etc.)
        // For now we mark as "—" unless the class JSON carries it; callers can extend this.
        "—".to_string()
    } else {
        "Loading…".to_string()
    };

    // ── Left column data ──
    let hit_die = format!("d{}", class.hit_die);
    let primary_ability = class
        .spellcasting_ability
        .as_deref()
        .map(|s| title_case(s))
        .unwrap_or_else(|| "—".to_string());
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

    // ── Right column data ──
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

    // ── Outer block ──
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" {} — Class Traits ", class.name))
        .border_style(Style::default().fg(Color::DarkGray));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // ── Split inner into two columns ──
    let cols =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).split(inner);

    // Left column
    let left_text = vec![
        Line::from(vec![
            Span::styled("Hit Die:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                hit_die,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Primary:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(primary_ability, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("Saves:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(saves, Style::default().fg(Color::White)),
        ]),
    ];

    // Right column
    let armor_display = truncate_str(&armor, cols[1].width.saturating_sub(12) as usize);
    let weapons_display = truncate_str(&weapons, cols[1].width.saturating_sub(12) as usize);
    let tool_display = truncate_str(&tool_profs, cols[1].width.saturating_sub(12) as usize);

    let right_text = vec![
        Line::from(vec![
            Span::styled("Armor:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(armor_display, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Weapons:  ", Style::default().fg(Color::DarkGray)),
            Span::styled(weapons_display, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Tools:    ", Style::default().fg(Color::DarkGray)),
            Span::styled(tool_display, Style::default().fg(Color::DarkGray)),
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
                        // Subclass features get a magenta tint; class features get cyan level marker
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

                    // Source badge for subclass rows
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
                    // Human-readable label for this decision type
                    let label_type = match choice_type.as_str() {
                        "asi" => "ASI / FEAT",
                        "weapon_mastery" => "WEAPON MASTERY",
                        "subclass" => "SUBCLASS",
                        other => other,
                    };

                    // ── Title row: "Lvl X │ Ability Score Improvement" ────────
                    let level_str = format!("Lvl {:>2} │ ", level);
                    let title_name = match choice_type.as_str() {
                        "asi" => "Ability Score Improvement".to_string(),
                        "weapon_mastery" => "Weapon Mastery".to_string(),
                        "subclass" => {
                            let sc_title = app
                                .class_detail
                                .as_ref()
                                .and_then(|d| d.class.subclass_title.clone())
                                .unwrap_or_else(|| "Subclass".to_string());
                            sc_title
                        }
                        other => title_case(other),
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

                    let title_line = Line::from(vec![
                        Span::styled(level_str, title_lvl_style),
                        Span::styled(title_name, title_name_style),
                    ]);

                    // ── Choice sub-row: "  ╰─ [ SELECT ASI/FEAT ]" ───────────
                    let slot_span = if is_locked {
                        Span::styled(
                            format!("  ╰─ [ LOCKED — {} ]", label_type),
                            Style::default().fg(Color::DarkGray),
                        )
                    } else {
                        match status {
                            DecisionStatus::Pending => {
                                let fg = if blink { Color::Yellow } else { Color::DarkGray };
                                Span::styled(
                                    format!("  ╰─ [ SELECT {} ]", label_type),
                                    Style::default().fg(fg).add_modifier(Modifier::BOLD),
                                )
                            }
                            DecisionStatus::Partial => Span::styled(
                                format!(
                                    "  ╰─ [ SELECT MORE ({}/{}) ]",
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
                                    format!("  ╰─ [ Selected: {} ]", desc_str),
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
        .highlight_style(
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(tree_list, area, &mut app.builder.feature_list_state);
}

// ── Decision helpers ──────────────────────────────────────────────────────────

pub fn has_unfulfilled_choices(app: &App) -> bool {
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx) {
            let rows = build_progression_rows(app, class);
            for r in &rows {
                if r.level() <= app.builder.level {
                    if let ProgressionRow::DecisionSlot { status, .. } = r {
                        if *status == DecisionStatus::Pending
                            || *status == DecisionStatus::Partial
                        {
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
    app.builder.progression_blink_tick =
        app.builder.progression_blink_tick.wrapping_add(1);

    let rows_count = if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx) {
            build_progression_rows(app, class).len()
        } else {
            0
        }
    } else {
        0
    };

    // ── Ctrl+K: feature detail modal ──────────────────────────────────────────
    if key.code == KeyCode::Char('k') && key.modifiers.contains(KeyModifiers::CONTROL) {
        if let Some(idx) = app.builder.list_state.selected() {
            if let Some(class) = app.classes.get(idx) {
                let rows = build_progression_rows(app, class);
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
                            let title =
                                format!("Level {} Feature{}: {}", level, src_label, name);
                            app.builder.feature_detail_modal =
                                Some((title, description.clone()));
                        }
                        ProgressionRow::DecisionSlot {
                            level,
                            choice_type,
                            descriptions,
                            status,
                            ..
                        } => {
                            let title = format!(
                                "Level {} Slot: {}",
                                level,
                                choice_type.to_uppercase()
                            );
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
                let len = app.classes.len();
                let i = match app.builder.list_state.selected() {
                    Some(0) | None => len.saturating_sub(1),
                    Some(i) => i - 1,
                };
                app.builder.list_state.select(Some(i));
                load_class_detail_for_selected(app);
            } else {
                app.builder.feature_cursor =
                    app.builder.feature_cursor.saturating_sub(1);
            }
        }
        KeyCode::Down => {
            if app.builder.focus_index == 0 {
                let len = app.classes.len();
                let i = match app.builder.list_state.selected() {
                    Some(i) if i + 1 < len => i + 1,
                    _ => 0,
                };
                app.builder.list_state.select(Some(i));
                load_class_detail_for_selected(app);
            } else if rows_count > 0 {
                app.builder.feature_cursor =
                    (app.builder.feature_cursor + 1).min(rows_count - 1);
            }
        }

        // Vim-style navigation always targets the right pane
        KeyCode::Char('k') | KeyCode::Char('K') => {
            app.builder.focus_index = 1;
            app.builder.feature_cursor =
                app.builder.feature_cursor.saturating_sub(1);
        }
        KeyCode::Char('j') | KeyCode::Char('J') => {
            app.builder.focus_index = 1;
            if rows_count > 0 {
                app.builder.feature_cursor =
                    (app.builder.feature_cursor + 1).min(rows_count - 1);
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
            }
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            if app.builder.level < 20 {
                app.builder.level += 1;
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
                    app.status_msg =
                        "No subclasses available for this class.".to_string();
                }
            } else {
                app.status_msg = format!(
                    "Reach level {} to pick a subclass (currently level {}).",
                    unlock,
                    app.builder.level
                );
            }
        }

        // Tab: proceed to next step
        KeyCode::Tab => {
            if has_unfulfilled_choices(app) {
                app.status_msg =
                    "Please complete all choices for your current level.".to_string();
                return;
            }
            if let Some(idx) = app.builder.list_state.selected() {
                if let Some(class) = app.classes.get(idx) {
                    app.builder.class_id = Some(class.id);
                    if !app.save_draft() {
                        return;
                    }
                    refresh_progression_manifest(app);
                    app.builder.step = CharacterCreationStep::Background;
                    app.builder.list_state.select(Some(0));
                    app.status_msg.clear();
                }
            }
        }

        // Enter: confirm class (left pane) or activate slot (right pane)
        KeyCode::Enter => {
            if app.builder.focus_index == 0 {
                // Confirm highlighted class and switch focus to progression tree
                if let Some(idx) = app.builder.list_state.selected() {
                    if let Some(class) = app.classes.get(idx) {
                        let class_id = class.id;
                        let caster = class.caster_progression.clone();

                        if app
                            .class_detail
                            .as_ref()
                            .map(|d| d.class.id != class_id)
                            .unwrap_or(true)
                        {
                            load_class_detail_for_selected(app);
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
                // Activate focused slot in progression tree
                if let Some(idx) = app.builder.list_state.selected() {
                    if let Some(class) = app.classes.get(idx) {
                        let rows = build_progression_rows(app, class);
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
                                } => match choice_type.as_str() {
                                    "asi" => {
                                        app.builder.show_progression_asi_modal = true;
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
                                },
                            }
                        }
                    }
                }
            }
        }

        _ => {}
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

fn load_class_detail_for_selected(app: &mut App) {
    if let Some(idx) = app.builder.list_state.selected() {
        if let Some(class) = app.classes.get(idx) {
            let name = class.name.clone();
            let source = class.source_slug.clone();
            let current_id = app.class_detail.as_ref().map(|d| d.class.id);
            if current_id != Some(class.id) {
                let rt = app.rt.clone();
                let client = app.client.clone();
                if let Ok(detail) = rt.block_on(client.get_class_detail(&name, &source)) {
                    app.class_detail = Some(detail);
                }
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
