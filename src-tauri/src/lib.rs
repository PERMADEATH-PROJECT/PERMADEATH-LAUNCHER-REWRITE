mod commands;
mod core;
mod database;
mod models;
mod services;

use models::options::{LauncherOptions, GameOptions};
use log::{info, error};
use database::DbManager;
use services::session_service::SessionService;
use services::options_repository::OptionsRepository;
use services::java_installer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
#[tokio::main]
pub async fn run() {
    let options = LauncherOptions::new();
    let game_options = GameOptions::new();

    // Logger setup
    if let Err(e) = core::logger::setup_logger(&options) {
        eprintln!("Error while setting up the logger: {}", e);
    }

    info!("Starting application");

    if !OptionsRepository::is_launcher_json_present(&options) {
        info!("Options file not found, creating a new one with default settings");
        OptionsRepository::save_launcher_options(&options);
    }

    if !OptionsRepository::is_game_json_present(&options) {
        info!("Game Options file not found, creating a new one with default settings");
        OptionsRepository::save_game_options(&game_options, &options);
    }

    // --- DATABASE CONNECTION ---
    dotenvy::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be defined .env");

    let db_manager = match DbManager::new(&db_url).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("We couldn't establish connection to the database: {}", e);
            return;
        }
    };

    // SessionService injected via tauri::State (Dependency Injection pattern)
    let session_service = match sqlx::MySqlPool::connect(&db_url).await {
        Ok(pool) => SessionService::new(pool),
        Err(e) => {
            error!("Error while creating the connection pool: {}", e);
            return;
        }
    };

    info!("Database connection established successfully.");

    let java_installed = java_installer::ensure_java_installed("21").await;

    if java_installed {
        info!("Setting up Tauri application");
        tauri::Builder::default()
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_process::init())
            .plugin(tauri_plugin_opener::init())
            .manage(db_manager)
            .manage(session_service)
            .invoke_handler(tauri::generate_handler![
                commands::options::read_options,
                commands::options::save_options,
                commands::options::return_default_game_dir,
                commands::options::read_game_options,
                commands::options::get_garbage_collectors,
                commands::options::get_base_jvm_flags,
                commands::options::save_game_options,
                commands::auth::login_user,
                commands::auth::register_user,
                commands::auth::check_session,
                commands::auth::logout,
                commands::auth::load_user_data,
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    } else {
        error!("Failed to install Java 21. The application cannot continue.");
        java_installer::show_info_message(
            "The application requires Java 21 to run. Please install Java 21 and try again."
        );
    }
}