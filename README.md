# CLI Adventure Sheet

A terminal-based D&D 5e character sheet manager built with Rust and [Ratatui](https://github.com/ratatui/ratatui).

## Features

- **Character Creation Wizard** — Step-by-step builder covering race, class, ability scores, background, equipment, and spells
- **Character Sheet** — Organized tabs for stats, skills, features, spells, inventory, background, and notes
- **Combat Tracking** — Death saves, spell slots, hit dice, conditions, and concentration
- **Compendium Browser** — Browse classes, races, spells, items, monsters, backgrounds, and feats
- **Multiclassing** — Full support for multiple classes per character
- **Inventory & Equipment** — Manage gear, weapons, armor, and items
- **Level Up** — XP tracking with ASI and feat selection
- **Authentication** — Account system for syncing characters across devices

## Supported Content

In this version the CLI app primarily supports:

- **Class** — Paladin and Tamer
- **Background** — Soldier and Sage
- **Species** — Human

## Requirements

- Rust 1.93.0+ (2024 edition)
- A running instance of the Adventure Sheets API backend

## Building

```bash
cargo build --release
```

To embed a custom API URL at compile time:

```bash
API_URL="https://your-server.com/api/v1" cargo build --release
```

If `API_URL` is not set, it defaults to `http://localhost:8080/api/v1`.

## Usage

```bash
cargo run
```

Or run the compiled binary directly:

```bash
./target/release/cli_adventure_sheet
```

## License

Business Source License 1.1 — converts to MIT on 2030-01-01. See [LICENSE](LICENSE) for details.

## Disclaimer

This project is an unofficial fan-made tool and is not affiliated with
Wizards of the Coast or D&D Beyond.

No copyrighted game content is included in this repository.
Users must provide their own JSON data files for races, classes,
spells, and other game data.
