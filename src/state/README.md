# State Management Module

This module provides a clean abstraction layer for game state persistence, designed to support both single-player (file-based) and multiplayer (SpacetimeDB) modes.

## Architecture Overview

```
┌─────────────────┐
│   Game Logic    │
│  (game/mod.rs)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  State Manager  │  ← Central access point
│ (state/mod.rs)  │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────────┐
│   Persistence Backend Trait     │
├─────────────────┬───────────────┤
│  FileBackend    │ SpacetimeDB   │ ← Swappable backends
│   (current)     │   (future)    │
└─────────────────┴───────────────┘
```

## Core Components

### 1. `Persistable` Trait

All game state types implement this trait:

```rust
pub trait Persistable: Serialize + for<'de> Deserialize<'de> {
    fn state_type() -> &'static str;
    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()>;
    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self>;
}
```

### 2. `PersistenceBackend` Trait

Abstraction for storage backends:

```rust
pub trait PersistenceBackend: Send + Sync {
    fn save(&self, key: &str, data: &[u8]) -> Result<()>;
    fn load(&self, key: &str) -> Result<Vec<u8>>;
    fn list(&self, state_type: &str) -> Result<Vec<String>>;
    fn delete(&self, key: &str) -> Result<()>;
    fn exists(&self, key: &str) -> Result<bool>;
}
```

### 3. State Types

#### `CharacterState`
Wraps `ForgeCharacter` with multiplayer-specific state:
- Character data (stats, inventory, equipment)
- Dungeon position (if in dungeon instance)
- Character progress (quests, discovered locations, etc.)
- Session data (connection info for multiplayer)

#### `WorldState`
Server-authoritative world state:
- World seed (ensures consistency across clients)
- Generated zones with settlements, POIs, roads, rivers
- Zone metadata (generated time, last visited)
- Campaign flags and world progress

#### `DungeonState`
Dungeon instance state:
- Instance ID and world location
- Floor layouts and tiles
- Enemy positions and health
- Loot instances
- Participants (for party dungeons)
- Progress tracking

#### `CombatState`
Turn-based tactical combat:
- Combat location (world or dungeon)
- All combatants (players and enemies)
- Initiative order and current turn
- Combat grid/battlefield
- Combat log for replay/debugging
- Status effects

## Current Implementation: FileBackend

The `FileBackend` stores each state as a JSON file:

```
.
├── character_HeroName.json
├── world_MainWorld.json
├── dungeon_instance123.json
└── combat_encounter456.json
```

## Usage Examples

### Basic State Management

```rust
use warlords::state::{StateManager, CharacterState};

// Initialize with file backend
let manager = StateManager::with_file_backend("./game_data")?;

// Save character state
let char_state = CharacterState::new(forge_character);
manager.save("player1", &char_state)?;

// Load character state
let loaded: CharacterState = manager.load("player1")?;

// List all characters
let character_ids = manager.list::<CharacterState>()?;
```

### Custom Backend

```rust
struct MyCustomBackend {
    // Your backend implementation
}

impl PersistenceBackend for MyCustomBackend {
    fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        // Your save logic
    }
    // ... implement other methods
}

let manager = StateManager::new(Box::new(MyCustomBackend::new()));
```

## Migration Path to SpacetimeDB

When ready for multiplayer, we'll implement `SpacetimeBackend`:

```rust
pub struct SpacetimeBackend {
    client: SpacetimeClient,
}

impl PersistenceBackend for SpacetimeBackend {
    fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        // Translate to SpacetimeDB table operations
        self.client.call_reducer("save_state", key, data)
    }

    fn load(&self, key: &str) -> Result<Vec<u8>> {
        // Query SpacetimeDB tables
        self.client.query("get_state", key)
    }

    // ... other methods
}
```

### SpacetimeDB Schema Design

Each state type will map to SpacetimeDB tables:

```rust
// Character table
#[spacetimedb::table(name = character_state)]
pub struct CharacterStateRow {
    #[primarykey]
    id: String,
    data: Vec<u8>,  // Serialized CharacterState
    updated_at: Timestamp,
}

// World table
#[spacetimedb::table(name = world_state)]
pub struct WorldStateRow {
    #[primarykey]
    id: String,
    seed: u64,
    data: Vec<u8>,  // Serialized WorldState
}

// Dungeon instances
#[spacetimedb::table(name = dungeon_state)]
pub struct DungeonStateRow {
    #[primarykey]
    instance_id: String,
    participants: Vec<String>,
    data: Vec<u8>,
}
```

## State Synchronization Strategy

### Single Player (FileBackend)
- Save on state changes (character level up, zone change, etc.)
- Auto-save every N minutes
- Load on game start

### Multiplayer (SpacetimeBackend)
- **CharacterState**: Client-initiated, server-validated
- **WorldState**: Server-authoritative, clients subscribe
- **DungeonState**: Instanced, party members subscribe
- **CombatState**: Server-authoritative turn resolution

## Benefits of This Architecture

1. **Separation of Concerns**: Game logic doesn't know about persistence details
2. **Testability**: Easy to mock backends for testing
3. **Flexibility**: Swap backends without changing game code
4. **Multiplayer Ready**: Designed with client-server in mind
5. **Type Safety**: Rust's type system ensures correctness
6. **Serialization**: Uses `serde` for flexible data formats

## Next Steps

Current phase (file-based):
- [x] Create state module structure
- [x] Define core traits
- [x] Implement all state types
- [x] Implement FileBackend
- [ ] Migrate game logic to use StateManager
- [ ] Add auto-save functionality

Future phase (multiplayer):
- [ ] Implement SpacetimeBackend
- [ ] Define SpacetimeDB schema
- [ ] Add real-time state synchronization
- [ ] Implement conflict resolution
- [ ] Add client-side prediction
- [ ] Network optimization (delta updates)

## File Organization

```
src/state/
├── mod.rs          # Core traits, StateManager, FileBackend
├── character.rs    # CharacterState definition
├── world.rs        # WorldState definition
├── dungeon.rs      # DungeonState definition
├── combat.rs       # CombatState definition
└── README.md       # This file
```

## Notes for Developers

- Always use `StateManager` to access state, never direct file I/O
- State types should be `Clone` to support rollback/undo
- Keep state serializable (avoid complex references)
- Session-specific data uses `#[serde(skip)]` to not persist
- When adding new state, implement `Persistable` trait
- Test with both `FileBackend` and mock backends

## Questions?

For questions about state management architecture, see the main game documentation or check the inline comments in the source files.
