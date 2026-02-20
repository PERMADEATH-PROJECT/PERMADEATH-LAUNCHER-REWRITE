use log::info;

use crate::models::options::{LauncherOptions, GameOptions, GarbageCollector, BASE_VM_FLAGS};
use crate::services::options_repository::OptionsRepository;

#[tauri::command]
pub fn read_options() -> LauncherOptions {
    info!("Loading options");
    let options = OptionsRepository::load_launcher_options();
    info!("Options loaded: {:?}", options);
    options
}

#[tauri::command]
pub fn save_options(options: LauncherOptions) -> bool {
    info!("Saving options: {:?}", options);
    OptionsRepository::save_launcher_options(&options);
    info!("Options saved correctly");
    true
}

#[tauri::command]
pub fn return_default_game_dir() -> String {
    info!("Obtaining default game directory");
    LauncherOptions::get_default_game_dir()
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[tauri::command]
pub fn read_game_options(launcher_options: LauncherOptions) -> GameOptions {
    info!("Loading game options");
    let game_options = OptionsRepository::load_game_options(&launcher_options);
    info!("Game options loaded: {:?}", game_options);
    game_options
}

#[tauri::command]
pub fn get_garbage_collectors() -> Vec<GarbageCollector> {
    info!("Retrieving garbage collectors");
    let collectors = GameOptions::get_garbage_collectors();
    info!("Garbage collectors retrieved: {:?}", collectors);
    collectors
}

#[tauri::command]
pub fn get_base_jvm_flags() -> Vec<String> {
    info!("Retrieving base JVM flags");
    let flags = BASE_VM_FLAGS.iter().map(|s| s.to_string()).collect();
    info!("Base JVM flags retrieved: {:?}", flags);
    flags
}

#[tauri::command]
pub fn save_game_options(game_options: GameOptions, launcher_options: LauncherOptions) -> bool {
    info!("Saving game options: {:?}", game_options);
    OptionsRepository::save_game_options(&game_options, &launcher_options);
    info!("Game options saved correctly");
    true
}
