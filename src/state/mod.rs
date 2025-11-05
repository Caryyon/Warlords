use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

// Re-export state modules
pub mod character;
pub mod world;
pub mod dungeon;
pub mod combat;
pub mod game_state;

pub use character::CharacterState;
pub use world::WorldState;
pub use dungeon::DungeonState;
pub use combat::CombatState;
pub use game_state::GameStateManager;

/// Core trait for all persistable game state
/// This abstraction allows us to swap persistence backends (file, SpacetimeDB, etc.)
pub trait Persistable: Serialize + for<'de> Deserialize<'de> {
    /// Unique identifier for this state type
    fn state_type() -> &'static str;

    /// Save this state to the current persistence backend
    fn save(&self, backend: &dyn PersistenceBackend) -> Result<()>;

    /// Load this state from the current persistence backend
    fn load(backend: &dyn PersistenceBackend, id: &str) -> Result<Self> where Self: Sized;
}

/// Abstraction for persistence backends
/// Implementations: FileBackend (current), SpacetimeBackend (future)
pub trait PersistenceBackend: Send + Sync {
    /// Save data with a given key
    fn save(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Load data by key
    fn load(&self, key: &str) -> Result<Vec<u8>>;

    /// List all keys of a certain type
    fn list(&self, state_type: &str) -> Result<Vec<String>>;

    /// Delete data by key
    fn delete(&self, key: &str) -> Result<()>;

    /// Check if key exists
    fn exists(&self, key: &str) -> Result<bool>;
}

/// File-based persistence backend (current implementation)
pub struct FileBackend {
    base_path: std::path::PathBuf,
}

impl FileBackend {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;
        Ok(Self { base_path })
    }

    fn get_path(&self, key: &str) -> std::path::PathBuf {
        self.base_path.join(format!("{}.json", key))
    }
}

impl PersistenceBackend for FileBackend {
    fn save(&self, key: &str, data: &[u8]) -> Result<()> {
        let path = self.get_path(key);
        std::fs::write(path, data)?;
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.get_path(key);
        let data = std::fs::read(path)?;
        Ok(data)
    }

    fn list(&self, state_type: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();

        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem.starts_with(state_type) {
                        keys.push(stem.to_string());
                    }
                }
            }
        }

        Ok(keys)
    }

    fn delete(&self, key: &str) -> Result<()> {
        let path = self.get_path(key);
        std::fs::remove_file(path)?;
        Ok(())
    }

    fn exists(&self, key: &str) -> Result<bool> {
        let path = self.get_path(key);
        Ok(path.exists())
    }
}

/// Central state manager
/// This will be the single point of access for all game state
pub struct StateManager {
    backend: Box<dyn PersistenceBackend>,
}

impl StateManager {
    pub fn new(backend: Box<dyn PersistenceBackend>) -> Self {
        Self { backend }
    }

    pub fn with_file_backend<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let backend = FileBackend::new(base_path)?;
        Ok(Self {
            backend: Box::new(backend),
        })
    }

    /// Save any persistable state
    pub fn save<T: Persistable>(&self, id: &str, state: &T) -> Result<()> {
        let key = format!("{}_{}", T::state_type(), id);
        let data = serde_json::to_vec_pretty(state)?;
        self.backend.save(&key, &data)
    }

    /// Load any persistable state
    pub fn load<T: Persistable>(&self, id: &str) -> Result<T> {
        let key = format!("{}_{}", T::state_type(), id);
        let data = self.backend.load(&key)?;
        let state = serde_json::from_slice(&data)?;
        Ok(state)
    }

    /// List all states of a given type
    pub fn list<T: Persistable>(&self) -> Result<Vec<String>> {
        self.backend.list(T::state_type())
    }

    /// Delete a state
    pub fn delete<T: Persistable>(&self, id: &str) -> Result<()> {
        let key = format!("{}_{}", T::state_type(), id);
        self.backend.delete(&key)
    }

    /// Check if a state exists
    pub fn exists<T: Persistable>(&self, id: &str) -> Result<bool> {
        let key = format!("{}_{}", T::state_type(), id);
        self.backend.exists(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires tempfile crate
    fn test_file_backend() -> Result<()> {
        // let temp_dir = tempfile::tempdir()?;
        // let backend = FileBackend::new(temp_dir.path())?;
        // backend.save("test_key", b"test data")?;
        // let data = backend.load("test_key")?;
        // assert_eq!(data, b"test data");
        // assert!(backend.exists("test_key")?);
        // backend.delete("test_key")?;
        // assert!(!backend.exists("test_key")?);
        Ok(())
    }
}
