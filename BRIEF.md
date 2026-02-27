# WARLORDS — Development Brief

## Project
Terminal-based MMORPG-style RPG using Forge: Out of Chaos rules.
Repo: /home/cwolff/Code/warlords
Language: Rust, ratatui (TUI), crossterm
Git remote: https://github.com/Caryyon/warlords

## Game Vision
Players start as a nobody — farmer, barkeep, blacksmith's apprentice, etc.
Through adventure they gain power, followers, and eventually become a WARLORD:
- Own villages, cities, castles
- Command armies and spies
- Political intrigue, territory control
- Persistent world shared across players (the network layer foundation exists)

## What Exists Already
- `src/forge/mod.rs` — Full Forge character system: characteristics (STR/STA/INT/INS/DEX/AWR/SPD/POW/LUC), races (Human, Elf, Dwarf, etc. with racial mods), inventory, weapons, armor
- `src/forge/combat.rs` — Turn-based combat with Forge rules: initiative, attack/defense values, damage, armor absorption, skill advancement
- `src/forge/magic.rs` — Magic/spell system (partial)
- `src/game/mod.rs` — Game loop, key bindings, game contexts (WorldExploration, DungeonExploration, CharacterCreation, Combat, etc.)
- `src/ui/` — ratatui UI: layout.rs, framework.rs, components.rs, mouse.rs
- `src/world/` — World generation: terrain, settlements, dungeons, rivers, roads, NPCs, persistence
- `src/state/` — Character state, dungeon position, progress tracking, faction relationships
- `src/network/` — Multiplayer foundation (exists but not our focus right now)
- `forge_rules_ocr_test.txt` — Full Forge: Out of Chaos rulebook OCR (16k lines) — READ THIS for rules

## What Needs to Be Built

### 1. Account / Signup System
- Simple username + password (stored locally in JSON/SQLite, not network yet)
- On first run: welcome screen → signup or login
- Store accounts in `~/.local/share/warlords/accounts/`

### 2. Character Creation Flow (ratatui TUI)
- Pick starting background/occupation: Farmer, Barkeep, Blacksmith, Merchant, Fisher, Stable Hand, etc.
- Background affects starting skills and flavor text
- Roll characteristics (Forge system already exists: ForgeCharacterCreation::roll_characteristics())
- Pick race (already exists: ForgeCharacterCreation::get_available_races())
- Name your character
- Confirm and begin tutorial

### 3. Tutorial — "The Call to Adventure"
- Player starts in their home village with their boring job
- Short story-driven intro: something happens (bandits raid, strange traveler arrives, family in debt, etc.)
- Scripted encounters that teach movement, talking to NPCs, basic combat
- Ends with player leaving their old life behind and entering the wider world
- Should feel like the start of an epic journey, not a chore

### 4. ASCII World Map + Travel
- Large procedurally generated world (already partially exists in src/world/)
- Player moves around an ASCII-drawn overworld map:
  - `.` grass plains
  - `^` mountains  
  - `T` forest
  - `~` water/rivers
  - `#` roads
  - `*` towns/settlements
  - `D` dungeons
  - `C` castles
- Fog of war — unexplored areas are dark
- Travel takes time (Forge speed stat)
- Encounters on the road (monsters, travelers, merchants)
- Enter settlements to talk to NPCs, buy/sell, get quests

### 5. Warlord Progression System
This is the core fantasy of the game. As players gain power:

**Tier 1 — Adventurer (starting):**
- Solo or small party
- Clearing dungeons, doing quests
- Building reputation in regions

**Tier 2 — Hired Sword / Captain:**
- Can recruit followers (1-10 people)
- Lead a small band of mercenaries
- Take contracts from local lords
- Own a base camp

**Tier 3 — Warlord:**
- Control a territory (village or small town)
- Maintain an army (50-500 soldiers)
- Manage resources (gold, food, morale)
- Spy network — send agents to gather intel or sabotage enemies
- Declare war on rival warlords

**Tier 4 — Lord/King:**
- Multiple cities and castles
- Noble title and political power
- Vassal relationships (other players or NPCs pledge loyalty)
- Trade routes and taxation
- Epic-scale wars

### 6. NPC Factions & Politics
- Major factions exist in the world (already partially in state/character.rs as faction_relationships)
- Reputation with each faction opens or closes options
- NPCs remember the player's actions

## Key Rules from Forge: Out of Chaos (from forge_rules_ocr_test.txt)
- Stats: STR, STA, INT, INS, DEX, AWR, SPD (1-5), POW (2-20), LUC (6-16)
- Skills increase through use (pips system — successful use awards pips, accumulate to level up)
- Combat: 1d20 + Attack Value vs Defense Value; damage = dice count absorbed by armor
- Magic: Spell Points (SPTS), power-based casting
- Races: Human, Elf, Half-Elf, Dwarf, Halfling, Gnome, and several unique Forge races
- Experience: awarded for overcoming challenges, spent to advance levels
- Levels grant HP increases and skill slot unlocks

## Technical Notes
- Commit as: Cary Wolff <boss@caryyon.com>
- Push to `main` branch (this is a personal project, not deploy-sensitive)
- Build with: `cargo build` — fix any warnings/errors you introduce
- The forge_rules_ocr_test.txt file is noisy OCR but contains real rules — grep through it
- ratatui version: 0.24 (see Cargo.toml)
- Use `cargo check` frequently to catch type errors early
- Characters persist to `characters.json` (see src/world/persistence.rs)

## Style/Feel
- This is a REAL game, not a demo. Make it fun and playable end-to-end
- TUI should feel like a polished ncurses app — clean panels, clear navigation
- ASCII art for the world map should be atmospheric
- Dialog should have personality — NPCs with names, flavor text on actions
- The tutorial should have heart — player's mom crying as they leave the farm, etc.
- Think Dwarf Fortress meets Crusader Kings meets old-school MUD

## Start Here
1. Read forge_rules_ocr_test.txt (grep for key mechanics — it's 16k lines of OCR)
2. Run `cargo check` to see current state of the codebase
3. Understand the existing UI flow in src/game/mod.rs and src/ui/
4. Build the signup → character creation → tutorial → world flow
5. Then add the warlord progression tier system
6. Commit working increments frequently

When completely finished with a solid playable build, run:
openclaw system event --text "Warlords: playable build complete — signup, char creation, tutorial, world travel, and warlord progression all implemented" --mode now
