use PortManagerBack::services::auth_service::AuthService;
use PortManagerBack::models::auth::{LoginRequest, CreateUserRequest};
use PortManagerBack::repositories::{UserRepository, SettingsRepository};
use PortManagerBack::AppState;
use rusqlite::Connection;
use dashmap::DashMap;
use std::sync::atomic::AtomicBool;
use bollard::Docker;
use std::sync::Arc;

async fn setup_test_state() -> Arc<AppState> {
    let conn_users = Connection::open_in_memory().unwrap();
    conn_users.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, username TEXT NOT NULL UNIQUE, password_hash TEXT NOT NULL)", []).unwrap();
    let conn_settings = Connection::open_in_memory().unwrap();
    conn_settings.execute("CREATE TABLE settings (key TEXT PRIMARY KEY, value TEXT NOT NULL)", []).unwrap();

    Arc::new(AppState {
        docker: Docker::connect_with_local_defaults().unwrap_or_else(|_| Docker::connect_with_socket_defaults().unwrap()),
        jwt_secret: "test_secret".to_string(),
        login_attempts: DashMap::new(),
        user_repo: UserRepository::new(conn_users),
        settings_repo: SettingsRepository::new(conn_settings),
        auth_required: AtomicBool::new(true),
    })
}

#[tokio::test]
async fn test_create_user_success() {
    let state = setup_test_state().await;
    let payload = CreateUserRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    let result = AuthService::create_user(&state, payload).await;
    assert!(result.is_ok());

    assert!(state.user_repo.user_exists("testuser").unwrap());
}

#[tokio::test]
async fn test_create_user_already_exists() {
    let state = setup_test_state().await;
    let payload = CreateUserRequest {
        username: "testuser".to_string(),
        password: "password123".to_string(),
    };

    AuthService::create_user(&state, payload.clone()).await.unwrap();
    let result = AuthService::create_user(&state, payload).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "ALREADY_EXISTS");
}

#[tokio::test]
async fn test_login_success() {
    let state = setup_test_state().await;
    let username = "loginuser".to_string();
    let password = "password123".to_string();

    AuthService::create_user(&state, CreateUserRequest {
        username: username.clone(),
        password: password.clone(),
    }).await.unwrap();

    let result = AuthService::login(&state, LoginRequest {
        username: username.clone(),
        password,
    }).await;

    assert!(result.is_ok());
    assert!(!result.unwrap().token.is_empty());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let state = setup_test_state().await;
    
    AuthService::create_user(&state, CreateUserRequest {
        username: "user".to_string(),
        password: "correct".to_string(),
    }).await.unwrap();

    let result = AuthService::login(&state, LoginRequest {
        username: "user".to_string(),
        password: "wrong".to_string(),
    }).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "INVALID_CREDENTIALS");
}

#[tokio::test]
async fn test_toggle_auth() {
    let state = setup_test_state().await;
    
    AuthService::toggle_auth(&state, false).await.unwrap();
    assert_eq!(state.auth_required.load(std::sync::atomic::Ordering::SeqCst), false);

    AuthService::toggle_auth(&state, true).await.unwrap();
    assert_eq!(state.auth_required.load(std::sync::atomic::Ordering::SeqCst), true);
}
