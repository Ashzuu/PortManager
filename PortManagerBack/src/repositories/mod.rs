use rusqlite::Connection;
use std::sync::Mutex;

pub struct UserRepository {
    pub db: Mutex<Connection>,
}

impl UserRepository {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Mutex::new(conn),
        }
    }

    pub fn get_password_hash(&self, username: &str) -> Result<Option<String>, String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        let hash: Option<String> = db.query_row(
            "SELECT password_hash FROM users WHERE username = ?",
            [username],
            |row| row.get(0)
        ).ok();

        Ok(hash)
    }

    pub fn user_exists(&self, username: &str) -> Result<bool, String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        let exists: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM users WHERE username = ?)",
            [username],
            |row| row.get(0)
        ).unwrap_or(false);

        Ok(exists)
    }

    pub fn create_user(&self, username: &str, password_hash: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        db.execute(
            "INSERT INTO users (username, password_hash) VALUES (?, ?)",
            [username, password_hash],
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }

    pub fn list_users(&self) -> Result<Vec<(i32, String)>, String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        let mut stmt = db.prepare("SELECT id, username FROM users")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;
        
        let user_iter = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| format!("Failed to query users: {}", e))?;

        let mut users = Vec::new();
        for user in user_iter {
            users.push(user.map_err(|e| format!("Error mapping user: {}", e))?);
        }

        Ok(users)
    }
}

pub struct SettingsRepository {
    pub db: Mutex<Connection>,
}

impl SettingsRepository {
    pub fn new(conn: Connection) -> Self {
        Self {
            db: Mutex::new(conn),
        }
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>, String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        let value: Option<String> = db.query_row(
            "SELECT value FROM settings WHERE key = ?",
            [key],
            |row| row.get(0)
        ).ok();

        Ok(value)
    }

    pub fn update_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let db = self.db.lock().map_err(|_| "Failed to lock database".to_string())?;
        
        db.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            [key, value],
        ).map_err(|e| format!("Database error: {}", e))?;

        Ok(())
    }
}
