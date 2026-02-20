use serde::Serialize;

/// Represents a row of the 'users' table
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: i32,
    pub minecraft_username: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize)]
pub struct UserData {
    pub status: bool,
    pub survived_days: i32,
    pub last_login: String,
    pub server_role: String,
}

#[derive(serde::Serialize)]
pub struct SessionInfo {
    pub user_id: i32,
    pub username: String,
}
