use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::models::auth::{LoginRequest, AuthResponse, CreateUserRequest, Claims, UserResponse};
use crate::api::auth::AuthError;
use jsonwebtoken::{encode, Header, EncodingKey};
use std::time::{SystemTime, UNIX_EPOCH};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier, PasswordHasher, SaltString},
    Argon2
};
use rand_core::OsRng;
use crate::AppState;

pub struct AuthService;

impl AuthService {
    pub async fn login(state: &Arc<AppState>, payload: LoginRequest) -> Result<AuthResponse, AuthError> {
        let username = payload.username.clone();
        
        // Vérification du Rate Limiting
        if let Some(attempt) = state.login_attempts.get(&username) {
            if attempt.count >= 3 {
                let elapsed = attempt.last_attempt.elapsed();
                let lockout_duration = std::time::Duration::from_secs(30);
                
                if elapsed < lockout_duration {
                    let remaining = lockout_duration.as_secs() - elapsed.as_secs();
                    return Err(AuthError {
                        message: format!("Trop de tentatives. Veuillez réessayer dans {} secondes.", remaining),
                        code: "TOO_MANY_REQUESTS".to_string(),
                        retry_after: Some(remaining),
                    });
                }
            }
        }

        tracing::debug!("Tentative de login pour: {}", username);

        let password_hash = state.user_repo.get_password_hash(&username).map_err(|e| {
            tracing::error!("Erreur Repo lors du login: {}", e);
            AuthError {
                message: e,
                code: "INTERNAL_ERROR".to_string(),
                retry_after: None,
            }
        })?;

        let is_valid = if let Some(hash) = password_hash {
            match PasswordHash::new(&hash) {
                Ok(parsed_hash) => {
                    let result = Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_ok();
                    if !result {
                        tracing::warn!("Mot de passe incorrect pour: {}", username);
                    }
                    result
                },
                Err(e) => {
                    tracing::error!("Format de hash invalide en base pour {}: {}", username, e);
                    false
                }
            }
        } else {
            false
        };

        if !is_valid {
            tracing::warn!("Tentative de connexion échouée pour l'utilisateur: {}", username);
            
            let mut entry = state.login_attempts.entry(username).or_insert(crate::LoginAttempt {
                count: 0,
                last_attempt: std::time::Instant::now(),
            });
            
            if entry.last_attempt.elapsed() > std::time::Duration::from_secs(60) {
                entry.count = 1;
            } else {
                entry.count += 1;
            }
            entry.last_attempt = std::time::Instant::now();

            return Err(AuthError {
                message: "Identifiants invalides.".to_string(),
                code: "INVALID_CREDENTIALS".to_string(),
                retry_after: None,
            });
        }

        state.login_attempts.remove(&username);
        tracing::info!("Connexion réussie pour l'utilisateur: {}", username);

        let expiration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + (2 * 3600);

        let claims = Claims {
            sub: payload.username.clone(),
            exp: expiration,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
        ).map_err(|_| AuthError {
            message: "Erreur lors de la génération du token.".to_string(),
            code: "INTERNAL_ERROR".to_string(),
            retry_after: None,
        })?;

        Ok(AuthResponse { token })
    }

    pub async fn create_user(state: &Arc<AppState>, payload: CreateUserRequest) -> Result<(), AuthError> {
        let exists = state.user_repo.user_exists(&payload.username).map_err(|e| AuthError {
            message: e,
            code: "INTERNAL_ERROR".to_string(),
            retry_after: None,
        })?;

        if exists {
            return Err(AuthError {
                message: "Cet utilisateur existe déjà.".to_string(),
                code: "ALREADY_EXISTS".to_string(),
                retry_after: None,
            });
        }

        let salt = SaltString::generate(&mut OsRng);
        let password_hash = Argon2::default()
            .hash_password(payload.password.as_bytes(), &salt)
            .map_err(|_| AuthError {
                message: "Erreur lors du hachage du mot de passe.".to_string(),
                code: "INTERNAL_ERROR".to_string(),
                retry_after: None,
            })?
            .to_string();

        state.user_repo.create_user(&payload.username, &password_hash).map_err(|e| AuthError {
            message: e,
            code: "INTERNAL_ERROR".to_string(),
            retry_after: None,
        })?;

        Ok(())
    }

    pub async fn toggle_auth(state: &Arc<AppState>, required: bool) -> Result<(), AuthError> {
        let value = if required { "true" } else { "false" };
        
        state.settings_repo.update_setting("auth_required", value).map_err(|e| AuthError {
            message: e,
            code: "INTERNAL_ERROR".to_string(),
            retry_after: None,
        })?;

        state.auth_required.store(required, Ordering::SeqCst);
        Ok(())
    }

    pub async fn list_users(state: &Arc<AppState>) -> Result<Vec<UserResponse>, AuthError> {
        let users = state.user_repo.list_users().map_err(|e| AuthError {
            message: e,
            code: "INTERNAL_ERROR".to_string(),
            retry_after: None,
        })?;

        Ok(users.into_iter().map(|(id, username)| UserResponse { id, username }).collect())
    }
}
