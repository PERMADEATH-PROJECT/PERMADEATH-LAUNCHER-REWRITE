use log::{info, error};

use crate::database::DbManager;
use crate::models::user::{SessionInfo, UserData};
use crate::services::auth_service;
use crate::services::session_service::SessionService;

#[tauri::command]
pub async fn login_user(
    username: String,
    password: String,
    db: tauri::State<'_, DbManager>,
    session_service: tauri::State<'_, SessionService>,
) -> Result<String, String> {
    auth_service::validate_login_input(&username, &password)?;

    info!("Attempting to authenticate user: {}", username);

    match db.get_user_by_username(&username).await {
        Ok(Some(user)) => {
            if auth_service::verify_password(&password, &user.password_hash)? {
                match session_service.create_session(user.id).await {
                    Ok(token) => {
                        info!("Successful login for '{}', token created", username);
                        Ok(token)
                    }
                    Err(e) => {
                        error!("Error creating session for '{}': {}", username, e);
                        Err("Error creating session.".to_string())
                    }
                }
            } else {
                info!("Incorrect password for user '{}'", username);
                Err("Invalid credentials.".to_string())
            }
        },
        Ok(None) => {
            info!("User '{}' not found.", username);
            Err("Invalid credentials.".to_string())
        },
        Err(e) => {
            error!("Database error during login: {}", e);
            Err("Invalid credentials.".to_string())
        }
    }
}

#[tauri::command]
pub async fn register_user(
    username: String,
    password: String,
    invite_code: String,
    db: tauri::State<'_, DbManager>,
) -> Result<String, String> {
    info!("Starting registration process for user: '{}'", username);

    auth_service::validate_register_input(&username, &password, &invite_code)?;

    let password_hash = auth_service::hash_password(password).await?;

    match db.create_user_with_invite(&username, &password_hash, &invite_code).await {
        Ok(new_id) => {
            let success_message = format!("User '{}' registered successfully with ID {}!", username, new_id);
            info!("{}", success_message);
            Ok(success_message)
        },
        Err(sqlx::Error::RowNotFound) => {
            Err("The invitation code is invalid or has already been used.".to_string())
        },
        Err(e) => {
            error!("Database error during registration of '{}': {}", username, e);
            Err("An error occurred while registering the account. Please contact support.".to_string())
        }
    }
}

#[tauri::command]
pub async fn check_session(
    session_service: tauri::State<'_, SessionService>,
) -> Result<Option<SessionInfo>, String> {
    match SessionService::get_token_from_keyring() {
        Ok(Some(token)) => {
            match session_service.validate_token(&token).await {
                Ok(Some((user_id, username))) => {
                    Ok(Some(SessionInfo { user_id, username }))
                },
                Ok(None) => Ok(None),
                Err(e) => Err(format!("Error validating session: {}", e))
            }
        }
        Ok(None) => {
            info!("No token saved in keyring");
            Ok(None)
        }
        Err(e) => {
            error!("Error reading token from keyring: {}", e);
            Ok(None)
        }
    }
}

#[tauri::command]
pub async fn logout(
    session_service: tauri::State<'_, SessionService>,
) -> Result<(), String> {
    if let Ok(Some(token)) = SessionService::get_token_from_keyring() {
        session_service.delete_session(&token)
            .await
            .map_err(|e| format!("Error closing session: {}", e))
    } else {
        info!("No active session to close");
        Ok(())
    }
}

#[tauri::command]
pub async fn load_user_data(
    username: String,
    db: tauri::State<'_, DbManager>,
) -> Result<UserData, String> {
    db.load_user_data(&username)
        .await
        .map_err(|e| format!("Error loading user data: {}", e))
}
