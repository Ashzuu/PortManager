use crate::{AppState, LoginAttempt};
use crate::repositories::{UserRepository, SettingsRepository};
use crate::services::docker_client::init_docker_client;
use bollard::Docker;
use dashmap::DashMap;
use rusqlite::Connection;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use argon2::{
    password_hash::{SaltString, PasswordHasher},
    Argon2
};
use rand_core::OsRng;

pub struct InitService;

impl InitService {
    pub fn init_app_state() -> Result<Arc<AppState>, Box<dyn std::error::Error>> {
        let jwt_secret: String = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_de_developpement".to_string());
        let docker_client: Docker = init_docker_client();

        let db_path: &str = "portmanager.db";
        let conn: Connection = Connection::open(db_path)?;

        Self::setup_database(&conn)?;

        let auth_required: bool = Self::get_auth_setting(&conn)?;

        // Création des repositories
        let user_repo: UserRepository = UserRepository::new(Connection::open(db_path)?);
        let settings_repo: SettingsRepository = SettingsRepository::new(Connection::open(db_path)?);

        Ok(Arc::new(AppState {
            docker: docker_client,
            jwt_secret,
            login_attempts: DashMap::<String, LoginAttempt>::new(),
            user_repo,
            settings_repo,
            auth_required: AtomicBool::new(auth_required),
        }))
    }

    fn setup_database(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL
            )",
            [],
        )?;

        let user_count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row: &rusqlite::Row<'_>| row.get(0))?;
        if user_count == 0 {
            // Génération dynamique du hash pour "password"
            let salt = SaltString::generate(&mut OsRng);
            let argon2 = Argon2::default();
            let password_hash = argon2.hash_password("password".as_bytes(), &salt)
                .map_err(|e| format!("Erreur de hachage: {}", e))?
                .to_string();

            conn.execute(
                "INSERT INTO users (username, password_hash) VALUES (?, ?)",
                ["admin", &password_hash],
            )?;
            tracing::info!("Utilisateur admin par défaut créé avec succès.");
        }

        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(())
    }

    fn get_auth_setting(conn: &Connection) -> Result<bool, rusqlite::Error> {
        let auth_required_str: String = conn.query_row(
            "SELECT value FROM settings WHERE key = 'auth_required'",
            [],
            |row: &rusqlite::Row<'_>| row.get(0)
        ).unwrap_or_else(|_| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('auth_required', 'true')",
                [],
            ).expect("Erreur lors de l'initialisation de auth_required");
            "true".to_string()
        });
        Ok(auth_required_str == "true")
    }
}
