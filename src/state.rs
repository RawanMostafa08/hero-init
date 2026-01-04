use serde::{Deserialize, Serialize};
use std::io::Result;
use std::path::{self, Path};
use std::{collections::HashSet, fs, io};

use crate::{execution, paths};

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct HeroState {
    pub completed_modules: HashSet<String>,
}

// Loads HeroState from disk
pub fn load_state() -> Result<HeroState> {
    load_state_with_path(paths::HERO_STATE_PATH)
}

fn load_state_with_path(path: &str) -> Result<HeroState> {
    let content = fs::read_to_string(path);

    match content {
        Ok(data) => {
            let state = serde_yaml::from_str(&data)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            Ok(state)
        }
        // file doesn't exist, no saved state yet, return default state
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(HeroState::default()),

        Err(e) => Err(e),
    }
}

// Saves HeroState to disk
pub fn save_state(state: &HeroState) -> Result<()> {
    let path = Path::new(&paths::HERO_STATE_PATH);
    save_state_with_path(state, &path)
}

fn save_state_with_path(state: &HeroState, path: &path::Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_yaml::to_string(state).map_err(io::Error::other)?;

    execution::write_file_atomic(path, &data, 0o600) // owner only
}

// Returns true if the passed module already ran, false otherwise
pub fn is_module_complete(state: &HeroState, module: &str) -> bool {
    state.completed_modules.contains(module)
}

// Marks the module as completed
pub fn mark_module_complete(state: &mut HeroState, module: &str) {
    state.completed_modules.insert(module.to_string());
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_load_state_default() {
        // No file exists
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.yaml");
        // Remove if exists
        let _ = fs::remove_file(&state_path);

        let state = load_state().unwrap();
        assert!(state.completed_modules.is_empty());
        assert_eq!(state, HeroState::default());
    }

    #[test]
    fn test_load_state_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.yaml");
        let sample_state = HeroState {
            completed_modules: HashSet::from(["metadata".to_string()]),
        };
        let yaml = serde_yaml::to_string(&sample_state).unwrap();
        fs::write(&state_path, yaml).unwrap();

        let state = load_state_with_path(state_path.to_str().unwrap()).unwrap();
        assert!(state.completed_modules.contains("metadata"));
        assert_eq!(state, sample_state);
    }

    #[test]
    fn test_save_state() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir.path().join("test_state.yaml");
        let mut state = HeroState::default();
        mark_module_complete(&mut state, "test_module");

        save_state_with_path(&state, &test_path).unwrap();
        assert!(!is_module_complete(&state, "unmarked"));
        assert!(is_module_complete(&state, "test_module"));
    }

    #[test]
    fn test_is_module_complete() {
        let mut state = HeroState::default();
        assert!(!is_module_complete(&state, "metadata"));
        mark_module_complete(&mut state, "metadata");
        assert!(is_module_complete(&state, "metadata"));
    }

    #[test]
    fn test_mark_module_complete() {
        let mut state = HeroState::default();
        mark_module_complete(&mut state, "network");
        assert!(state.completed_modules.contains("network"));
    }

    #[test]
    fn test_load_state_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let state_path = temp_dir.path().join("state.yaml");
        fs::write(&state_path, "invalid: yaml").unwrap();
        // Again, path issue
        let result = load_state_with_path(&state_path.to_str().unwrap());
        // Expect error
        assert!(result.is_err());
    }
}
