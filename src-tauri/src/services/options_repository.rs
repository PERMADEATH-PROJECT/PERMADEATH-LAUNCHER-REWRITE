use std::fs::{write, create_dir_all};
use log::{info, error};

use crate::models::options::{LauncherOptions, GameOptions};

/// Handles all file I/O operations for options (Repository Pattern)
pub struct OptionsRepository;

impl OptionsRepository {
    pub fn save_launcher_options(options: &LauncherOptions) {
        if let Some(dir) = &options.launcher_dir {
            let options_path = dir.join("options.json");
            info!("Trying to save options at: {:?}", options_path);
            let json = match serde_json::to_string_pretty(options) {
                Ok(j) => j,
                Err(_) => {
                    error!("Could not serialize options to JSON.");
                    return;
                }
            };
            if let Err(e) = create_dir_all(dir) {
                error!("Failed to create config directory: {}", e);
                return;
            }
            info!("Config directory created or already exists: {:?}", dir);
            if let Err(e) = write(&options_path, json) {
                error!("Failed to write options file: {}", e);
                return;
            }
            info!("Options saved successfully at: {:?}", options_path);
            return;
        }
        info!("Launcher directory is not configured.");
    }

    pub fn load_launcher_options() -> LauncherOptions {
        let default_options = LauncherOptions::new();
        if let Some(dir) = &default_options.launcher_dir {
            let options_path = dir.join("options.json");
            info!("Trying to load options from: {:?}", options_path);
            let data = match std::fs::read_to_string(&options_path) {
                Ok(d) => d,
                Err(_) => {
                    info!("Options file not found, using default values.");
                    return default_options;
                }
            };
            info!("Options file found, trying to deserialize...");
            let options = match serde_json::from_str::<LauncherOptions>(&data) {
                Ok(o) => o,
                Err(_) => {
                    error!("Could not deserialize options file.");
                    return default_options;
                }
            };
            info!("Options loaded successfully.");
            return options;
        }
        info!("Launcher directory is not configured, using default values.");
        default_options
    }

    pub fn is_launcher_json_present(options: &LauncherOptions) -> bool {
        if let Some(dir) = &options.launcher_dir {
            let options_path = dir.join("options.json");
            let exists = options_path.exists();
            info!("Is the options file present? {} at {:?}", exists, options_path);
            return exists;
        }
        info!("Launcher directory is not configured to check the file.");
        false
    }

    pub fn save_game_options(game_options: &GameOptions, launcher_options: &LauncherOptions) {
        info!("Saving game options: {:?}", game_options);
        if let Some(dir) = &launcher_options.launcher_dir {
            let options_path = dir.join("game_options.json");
            let json = match serde_json::to_string_pretty(game_options) {
                Ok(j) => j,
                Err(e) => {
                    error!("Failed to serialize game options: {}", e);
                    return;
                }
            };
            if let Err(e) = create_dir_all(dir) {
                error!("Failed to create config directory: {}", e);
                return;
            }
            info!("Config directory created or already exists.");
            if let Err(e) = write(&options_path, json) {
                error!("Failed to write game options file: {}", e);
                return;
            }
            info!("Game options saved successfully at: {:?}", options_path);
            return;
        }
        info!("Launcher directory is not configured.");
    }

    pub fn load_game_options(launcher_options: &LauncherOptions) -> GameOptions {
        if let Some(dir) = &launcher_options.launcher_dir {
            let options_path = dir.join("game_options.json");
            info!("Trying to load game options from: {:?}", options_path);
            if let Ok(data) = std::fs::read_to_string(&options_path) {
                match serde_json::from_str::<GameOptions>(&data) {
                    Ok(options) => {
                        info!("Game options loaded successfully: {:?}", options);
                        return options;
                    }
                    Err(e) => {
                        error!("Failed to parse game options JSON: {}", e);
                    }
                }
            } else {
                info!("Game options file does not exist, using defaults.");
            }
        } else {
            info!("Launcher directory is not configured, using default game options.");
        }
        GameOptions::new()
    }

    pub fn is_game_json_present(launcher_options: &LauncherOptions) -> bool {
        if let Some(dir) = &launcher_options.launcher_dir {
            let options_path = dir.join("game_options.json");
            let exists = options_path.exists();
            info!("Game options JSON presence: {}", exists);
            return exists;
        }
        info!("Launcher directory is not configured, game options JSON cannot be present.");
        false
    }
}
