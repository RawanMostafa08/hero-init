use serde::{Deserialize, Serialize};
use std::io::Result;
use std::path::Path;
use std::{collections::HashSet, fs, io};

use crate::{execution, paths};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct HeroState {
    pub completed_modules: HashSet<String>,
}

// Loads HeroState from disk
pub fn load_state() -> Result<HeroState> {
    let content = fs::read_to_string(paths::HERO_STATE_PATH);

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
