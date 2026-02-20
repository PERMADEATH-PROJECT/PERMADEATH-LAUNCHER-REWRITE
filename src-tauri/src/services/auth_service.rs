use log::error;

/// Validates login input fields
pub fn validate_login_input(username: &str, password: &str) -> Result<(), String> {
    if username.is_empty() || username.len() > 16 {
        error!("Login attempt with invalid username length");
        return Err("Invalid credentials.".to_string());
    }

    if password.is_empty() || password.len() > 128 {
        error!("Login attempt with invalid password length");
        return Err("Invalid credentials.".to_string());
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        error!("Login attempt with invalid characters in username");
        return Err("Invalid credentials.".to_string());
    }

    Ok(())
}

/// Validates registration input fields
pub fn validate_register_input(username: &str, password: &str, invite_code: &str) -> Result<(), String> {
    if username.len() < 3 || username.len() > 16 {
        return Err("Username must be between 3 and 16 characters.".to_string());
    }

    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers and underscores.".to_string());
    }

    if password.len() < 8 {
        return Err("Password must be at least 8 characters.".to_string());
    }

    if password.len() > 128 {
        return Err("Password cannot exceed 128 characters.".to_string());
    }

    if invite_code.is_empty() || invite_code.len() > 64 {
        return Err("Invalid invitation code.".to_string());
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
            "An unexpected error occurred. Please try again.".to_string()
        })?
        .map_err(|e| {
            error!("Password hashing error: {}", e);
            "An unexpected error occurred. Please try again.".to_string()
        })
}

/// Verify a password against a bcrypt hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    bcrypt::verify(password, hash)
        .map_err(|e| {
            error!("Error verifying password: {}", e);
            "Invalid credentials.".to_string()
        })
}
