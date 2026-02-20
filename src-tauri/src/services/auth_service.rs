use log::error;

/// Validates login input fields
pub fn validate_login_input(username: &str, password: &str) -> Result<(), String> {
    if username.is_empty() || username.len() > 16 {
        error!("Login attempt with invalid username length");
        return Err("Credenciales inválidas.".to_string());
    }

    if password.is_empty() || password.len() > 128 {
        error!("Login attempt with invalid password length");
        return Err("Credenciales inválidas.".to_string());
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        error!("Login attempt with invalid characters in username");
        return Err("Credenciales inválidas.".to_string());
    }

    Ok(())
}

/// Validates registration input fields
pub fn validate_register_input(username: &str, password: &str, invite_code: &str) -> Result<(), String> {
    if username.len() < 3 || username.len() > 16 {
        return Err("El nombre de usuario debe tener entre 3 y 16 caracteres.".to_string());
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("El nombre de usuario solo puede contener letras, números y guiones bajos.".to_string());
    }

    if password.len() < 8 {
        return Err("La contraseña debe tener al menos 8 caracteres.".to_string());
    }

    if password.len() > 128 {
        return Err("La contraseña no puede tener más de 128 caracteres.".to_string());
    }

    if invite_code.is_empty() || invite_code.len() > 64 {
        return Err("Código de invitación inválido.".to_string());
    }

    Ok(())
}

/// Hash a password using bcrypt in a blocking thread
pub async fn hash_password(password: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }).await
        .map_err(|e| {
            error!("Hashing task failed: {}", e);
            "Ocurrió un error inesperado. Por favor, inténtalo de nuevo.".to_string()
        })?
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            "Ocurrió un error inesperado. Por favor, inténtalo de nuevo.".to_string()
        })
}

/// Verify a password against a bcrypt hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash)
        .map_err(|e| {
            error!("Error verifying password: {}", e);
            "Credenciales inválidas.".to_string()
        })
}
