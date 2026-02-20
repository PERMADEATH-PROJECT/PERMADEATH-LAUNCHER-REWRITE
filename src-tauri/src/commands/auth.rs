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

    info!("Intentando autenticar al usuario: {}", username);

    match db.get_user_by_username(&username).await {
        Ok(Some(user)) => {
            if auth_service::verify_password(&password, &user.password_hash)? {
                match session_service.create_session(user.id).await {
                    Ok(token) => {
                        info!("Login exitoso para '{}', token creado", username);
                        Ok(token)
                    }
                    Err(e) => {
                        error!("Error creando sesión para '{}': {}", username, e);
                        Err("Error al crear sesión.".to_string())
                    }
                }
            } else {
                info!("Incorrect password for user '{}'", username);
                Err("Credenciales inválidas.".to_string())
            }
        },
        Ok(None) => {
            info!("Usuario '{}' no encontrado.", username);
            Err("Credenciales inválidas.".to_string())
        },
        Err(e) => {
            error!("Error de base de datos durante el login: {}", e);
            Err("Credenciales inválidas.".to_string())
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
    info!("Iniciando proceso de registro para el usuario: '{}'", username);

    auth_service::validate_register_input(&username, &password, &invite_code)?;

    let password_hash = auth_service::hash_password(password).await?;

    match db.create_user_with_invite(&username, &password_hash, &invite_code).await {
        Ok(new_id) => {
            let success_message = format!("¡Usuario '{}' registrado con éxito con el ID {}!", username, new_id);
            info!("{}", success_message);
            Ok(success_message)
        },
        Err(sqlx::Error::RowNotFound) => {
            Err("El código de invitación no es válido o ya ha sido utilizado.".to_string())
        },
        Err(e) => {
            error!("Error de base de datos durante el registro de '{}': {}", username, e);
            Err("Ocurrió un error al registrar la cuenta. Por favor, contacta con soporte.".to_string())
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
                Err(e) => Err(format!("Error validando sesión: {}", e))
            }
        }
        Ok(None) => {
            info!("No hay token guardado en el keyring");
            Ok(None)
        }
        Err(e) => {
            error!("Error leyendo token del keyring: {}", e);
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
            .map_err(|e| format!("Error cerrando sesión: {}", e))
    } else {
        info!("No hay sesión activa para cerrar");
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
        .map_err(|e| format!("Error cargando datos del usuario: {}", e))
}
