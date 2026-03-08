use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn render(app: &App, frame: &mut Frame, area: Rect) {
    let character = match &app.active_character {
        Some(c) => c,
        None => return,
    };

    let chunks = Layout::vertical([
        Constraint::Length(4), // combat bar (AC / Init / Speed / HP / Hit Dice)
        Constraint::Length(4), // HP detail + Inspiration + Death saves
        Constraint::Length(1), // abilities header
        Constraint::Min(3),    // abilities 2-column grid (3 rows each)
        Constraint::Length(1), // saves header
        Constraint::Length(1), // saves
        Constraint::Length(1), // senses header
        Constraint::Length(3), // senses
        Constraint::Length(1), // conditions header
        Constraint::Min(1),    // conditions
    ])
    .split(area);

    let level = crate::utils::level_from_xp(character.experience_pts);
    let prof_bonus = crate::utils::proficiency_bonus(level);

    let dex_mod = crate::utils::ability_modifier(character.dexterity);
    let wis_mod = crate::utils::ability_modifier(character.wisdom);

    let ac = app.calc_ac(dex_mod);

    // Alert feat (2024 PHB): "Initiative Proficiency" — add proficiency bonus to initiative.
    let has_alert = app.char_feats.iter().any(|cf| {
        app.all_feats
            .iter()
            .find(|f| f.id == cf.feat_id)
            .map(|f| f.name.eq_ignore_ascii_case("alert"))
            .unwrap_or(false)
    });
    let initiative = dex_mod + if has_alert { prof_bonus } else { 0 };

    let speed = app.race_speed();

    let hit_die = app
        .classes
        .iter()
        .find(|c| c.name == app.char_class_name)
        .map(|c| c.hit_die)
        .unwrap_or(8);

    let hit_die_idx = match hit_die {
        6 => 0,
        8 => 1,
        10 => 2,
        12 => 3,
        __ => 1,
    };
    let hd_used = app.hit_dice_used[hit_die_idx];
    let hd_str = format!(
        "d{} ({}/{}) [h]  [S]hort/[L]ong Rest",
        hit_die,
        level - (hd_used as i32),
        level
    );

    // ── Combat Bar ──────────────────────────────────────────────────────────
    let combat_cols = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .split(chunks[0]);

    let stat_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let label_style = Style::default().fg(Color::DarkGray);

    for (i, (label, value)) in [
        ("  AC", format!("{ac}")),
        ("  INIT", crate::utils::format_modifier(initiative)),
        ("  SPEED", format!("{speed} ft")),
        ("  HIT DIE", hd_str),
    ]
    .iter()
    .enumerate()
    {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));
        let inner = block.inner(combat_cols[i]);
        frame.render_widget(block, combat_cols[i]);
        let para = Paragraph::new(vec![
            Line::from(Span::styled(label.to_string(), label_style)),
            Line::from(Span::styled(value.to_string(), stat_style)),
        ]);
        frame.render_widget(para, inner);
    }

    // ── HP + Inspiration + Death Saves ───────────────────────────────────────
    let hp_row = Layout::horizontal([
        Constraint::Percentage(50),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .split(chunks[1]);

    // HP block
    let hp_color = if character.current_hp <= character.max_hp / 4 {
        Color::Red
    } else if character.current_hp <= character.max_hp / 2 {
        Color::Yellow
    } else {
        Color::Green
    };
    let hp_text = if character.temp_hp > 0 {
        format!(
            "{}/{} (+{} temp)",
            character.current_hp, character.max_hp, character.temp_hp
        )
    } else {
        format!("{}/{}", character.current_hp, character.max_hp)
    };
    let hp_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" HP ", label_style))
        .border_style(Style::default().fg(Color::DarkGray));
    let hp_inner = hp_block.inner(hp_row[0]);
    frame.render_widget(hp_block, hp_row[0]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("  {hp_text}"),
            Style::default().fg(hp_color).add_modifier(Modifier::BOLD),
        )),
        hp_inner,
    );

    // Inspiration block
    let insp_color = if character.inspiration {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let insp_text = if character.inspiration {
        "★ INSPIRED"
    } else {
        "☆ No Insp."
    };
    let insp_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Inspiration [i] ", label_style))
        .border_style(Style::default().fg(Color::DarkGray));
    let insp_inner = insp_block.inner(hp_row[1]);
    frame.render_widget(insp_block, hp_row[1]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("  {insp_text}"),
            Style::default().fg(insp_color).add_modifier(Modifier::BOLD),
        )),
        insp_inner,
    );

    // Death saves block — 6 dots: ● ● ● (green saves)  ● ● ● (red fails)
    let dead = character.current_hp == 0;
    let ds_block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Death Saves ", label_style))
        .border_style(Style::default().fg(if dead { Color::Red } else { Color::DarkGray }));
    let ds_inner = ds_block.inner(hp_row[2]);
    frame.render_widget(ds_block, hp_row[2]);

    let mut dot_spans: Vec<Span> = vec![Span::raw(" ")];
    // 3 save dots (green)
    for i in 0..3u8 {
        let dot = if i < app.death_saves_success {
            "● "
        } else {
            "○ "
        };
        dot_spans.push(Span::styled(dot, Style::default().fg(Color::Green)));
    }
    dot_spans.push(Span::styled(" | ", Style::default().fg(Color::DarkGray)));
    // 3 fail dots (red)
    for i in 0..3u8 {
        let dot = if i < app.death_saves_fail {
            "● "
        } else {
            "○ "
        };
        dot_spans.push(Span::styled(dot, Style::default().fg(Color::Red)));
    }

    let label_line = Line::from(vec![
        Span::styled(" [s]✓ saves  ", Style::default().fg(Color::Green)),
        Span::styled("[d]✗ fails", Style::default().fg(Color::Red)),
    ]);

    frame.render_widget(
        Paragraph::new(vec![Line::from(dot_spans), label_line]),
        ds_inner,
    );

    // ── Abilities ────────────────────────────────────────────────────────────
    let abilities_header = Paragraph::new(Span::styled(
        "  ABILITIES",
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(abilities_header, chunks[2]);

    let scores = [
        ("STR", character.strength),
        ("DEX", character.dexterity),
        ("CON", character.constitution),
        ("INT", character.intelligence),
        ("WIS", character.wisdom),
        ("CHA", character.charisma),
    ];

    // Render abilities in 2 columns: STR/DEX/CON on left, INT/WIS/CHA on right
    let ability_cols = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[3]);

    for col in 0..2 {
        let col_scores = &scores[col * 3..(col + 1) * 3];
        let lines: Vec<Line> = col_scores
            .iter()
            .map(|(name, score)| {
                let m = crate::utils::ability_modifier(*score);
                let sign = if m >= 0 { "+" } else { "" };
                Line::from(vec![
                    Span::styled(
                        format!("  {name:<3}  "),
                        Style::default().fg(Color::DarkGray),
                    ),
                    Span::styled(
                        format!("{score:<2}"),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!("  ({}{m} mod)", sign),
                        Style::default().fg(Color::DarkGray),
                    ),
                ])
            })
            .collect();
        frame.render_widget(Paragraph::new(lines), ability_cols[col]);
    }

    // ── Saving Throws ────────────────────────────────────────────────────────
    let saves_header = Paragraph::new(Span::styled(
        "  SAVING THROWS",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(saves_header, chunks[4]);

    // Fix: match by class name, not first class found
    let prof_saves: Vec<String> = app
        .classes
        .iter()
        .find(|c| c.name == app.char_class_name)
        .and_then(|c| c.proficiency_saves.clone())
        .unwrap_or_default();

    let save_spans: Vec<Span> = scores
        .iter()
        .flat_map(|(name, score)| {
            let base_mod = crate::utils::ability_modifier(*score);
            let is_proficient = prof_saves.iter().any(|s| s.eq_ignore_ascii_case(name));
            let total = if is_proficient {
                base_mod + prof_bonus
            } else {
                base_mod
            };
            let color = if is_proficient {
                Color::Green
            } else {
                Color::White
            };
            let marker = if is_proficient { "●" } else { " " };
            vec![Span::styled(
                format!(" {marker}{name} {} ", crate::utils::format_modifier(total)),
                Style::default().fg(color),
            )]
        })
        .collect();

    let saves_line = Paragraph::new(Line::from(save_spans));
    frame.render_widget(saves_line, chunks[5]);

    // ── Senses ───────────────────────────────────────────────────────────────
    let senses_header = Paragraph::new(Span::styled(
        "  SENSES",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(senses_header, chunks[6]);

    // Passive Perception: 10 + WIS mod + prof bonus if proficient in Perception
    let perc_prof =
        prof_saves.iter().any(|s| s.eq_ignore_ascii_case("WIS")) || app.has_perception_prof();
    let invest_prof = app.has_skill_prof("investigation");
    let int_mod = crate::utils::ability_modifier(character.intelligence);
    let passive_perception = 10 + wis_mod + if perc_prof { prof_bonus } else { 0 };
    let passive_insight = 10 + wis_mod;
    let passive_investigation = 10 + int_mod + if invest_prof { prof_bonus } else { 0 };

    let senses_text = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Passive Perception:    ", label_style),
            Span::styled(
                format!("{passive_perception}"),
                Style::default().fg(Color::White),
            ),
            if perc_prof {
                Span::styled("  (proficient)", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(vec![
            Span::styled("  Passive Insight:       ", label_style),
            Span::styled(
                format!("{passive_insight}"),
                Style::default().fg(Color::White),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Passive Investigation: ", label_style),
            Span::styled(
                format!("{passive_investigation}"),
                Style::default().fg(Color::White),
            ),
            if invest_prof {
                Span::styled("  (proficient)", Style::default().fg(Color::Green))
            } else {
                Span::raw("")
            },
        ]),
    ]);
    frame.render_widget(senses_text, chunks[7]);

    // ── Conditions ───────────────────────────────────────────────────────────
    let cond_header = Paragraph::new(Span::styled(
        "  CONDITIONS  [c toggle]",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(cond_header, chunks[8]);

    let cond_text = if app.conditions.is_empty() {
        Paragraph::new(Span::styled("  None", Style::default().fg(Color::DarkGray)))
    } else {
        let spans: Vec<Span> = app
            .conditions
            .iter()
            .flat_map(|c| {
                [Span::styled(
                    format!("  ⚠ {c}"),
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )]
            })
            .collect();
        Paragraph::new(Line::from(spans))
    };
    frame.render_widget(cond_text, chunks[9]);
}
