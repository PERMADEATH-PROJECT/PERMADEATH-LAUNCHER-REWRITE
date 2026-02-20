use sqlx::{Error, MySqlPool};
use chrono::Utc;
use log::{info, error};

use crate::models::user::{User, UserData};

/// Main SQL Manager
pub struct DbManager {
    pool: MySqlPool,
}

impl DbManager {
    /// Create a new instance and connect to the database.
    pub async fn new(database_url: &str) -> Result<Self, Error> {
        let pool = MySqlPool::connect(database_url).await?;
        info!("Connection pool created successfully.");
        Ok(Self { pool })
    }

    // --- SELECT METHODS ---

    /// Search a user by their minecraft username.
    pub async fn get_user_by_username(&self, username: &str) -> Result<Option<User>, Error> {
        info!("Looking up user by name: '{}'", username);
        sqlx::query_as::<_, User>("SELECT id, minecraft_username, password_hash FROM users WHERE minecraft_username = ?")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
    }

    // --- INSERT METHODS ---

    /// Create a new user using an invitation code
    /// Execute a secure transaction
    pub async fn create_user_with_invite(
        &self,
        username: &str,
        password_hash: &str,
        invite_code: &str,
    ) -> Result<u64, Error> {
        info!("Starting transaction to register user: '{}'", username);
        let mut tx = self.pool.begin().await?;

        // Verify that the code is not used
        let invite_row: Option<(i32, bool)> = sqlx::query_as("SELECT id, claimed FROM invites WHERE code = ?")
            .bind(invite_code)
            .fetch_optional(&mut *tx)
            .await?;

        if invite_row.is_none() || invite_row.unwrap().1 {
            error!("Registration attempt failed: invitation code '{}' is invalid or already used.", invite_code);
            return Err(Error::RowNotFound);
        }

        // Insert the new user and obtain its id
        let new_user_id = sqlx::query("INSERT INTO users (minecraft_username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(password_hash)
            .execute(&mut *tx)
            .await?
            .last_insert_id();

        // Update the invitation to link it to the user
        sqlx::query("UPDATE invites SET claimed = TRUE, user_id = ? WHERE code = ?")
            .bind(new_user_id)
            .bind(invite_code)
            .execute(&mut *tx)
            .await?;

        // Create an entry in the `account_status` table
        sqlx::query("INSERT INTO account_status (user_id, last_connection) VALUES (?, ?)")
            .bind(new_user_id)
            .bind(Utc::now())
            .execute(&mut *tx)
            .await?;

        // Confirm the transaction
        tx.commit().await?;

        info!("Transaction completed. User '{}' created with ID: {}", username, new_user_id);
        Ok(new_user_id)
    }

    // --- UPDATE METHODS ---

    /// Update last connection date for a user by their ID.
    pub async fn update_user_last_connection(&self, user_id: i32) -> Result<u64, Error> {
        info!("Updating last connection for user with ID: {}", user_id);

        let result = sqlx::query!(
            "UPDATE account_status SET last_connection = ? WHERE user_id = ?",
            Utc::now(),
            user_id
        )
            .execute(&self.pool)
            .await?;

        info!("{} rows affected when updating connection for user {}.", result.rows_affected(), user_id);
        Ok(result.rows_affected())
    }

    pub async fn load_user_data(&self, username: &str) -> Result<UserData, Error> {
        info!("Loading user data for: '{}'", username);

        let result = sqlx::query!(
            r#"
            SELECT
                u.id,
                a.player_status,
                a.days_survived,
                a.last_connection
            FROM users u
            INNER JOIN account_status a ON u.id = a.user_id
            WHERE u.minecraft_username = ?
            "#,
            username
        )
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(row) => {
                let last_login = row.last_connection
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Never".to_string());

                Ok(UserData {
                    status: row.player_status.unwrap_or(0) != 0,
                    survived_days: row.days_survived.unwrap_or(0),
                    last_login,
                    server_role: "Player".to_string(),
                })

            }
            None => Err(Error::RowNotFound),
        }
    }
}
