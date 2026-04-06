use log::info;
use tauri::AppHandle;

use crate::services::game_service::{GameManager, GameState};
use crate::services::options_repository::OptionsRepository;
use crate::services::server_status_service::{self, ServerStatus};

/// Returns true if the CLI's config.properties already has a stored Microsoft token,
/// meaning the user was previously authenticated and won't need to open the browser again.
#[tauri::command]
pub fn check_ms_auth_state() -> bool {
    let opts = OptionsRepository::load_launcher_options();
    GameManager::has_ms_token(&opts)
}

#[tauri::command]
pub async fn launch_game(
    username: String,
    use_microsoft: bool,
    app: AppHandle,
    game_manager: tauri::State<'_, GameManager>,
) -> Result<(), String> {
    info!("launch_game: user='{}' microsoft={}", username, use_microsoft);
    game_manager.launch(username, use_microsoft, app).await
}

#[tauri::command]
pub async fn stop_game(
    game_manager: tauri::State<'_, GameManager>,
) -> Result<(), String> {
    info!("stop_game invoked");
    game_manager.kill().await;
    Ok(())
}

#[tauri::command]
pub fn get_game_state(
    game_manager: tauri::State<'_, GameManager>,
) -> GameState {
    game_manager.get_state()
}

#[tauri::command]
pub async fn get_server_status() -> ServerStatus {
    let host = std::env::var("MINECRAFT_SERVER_IP")
        .unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = std::env::var("MINECRAFT_SERVER_PORT")
        .unwrap_or_else(|_| "25565".to_string())
        .parse()
        .unwrap_or(25565);
    info!("Pinging server at {}:{}", host, port);
    server_status_service::get_server_status(&host, port).await
}
