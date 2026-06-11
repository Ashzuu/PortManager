use bollard::Docker;
use dashmap::DashMap;
use std::time::Instant;
use std::sync::atomic::AtomicBool;
use crate::repositories::{UserRepository, SettingsRepository};

pub mod api;
pub mod models;
pub mod services;
pub mod repositories;

pub struct LoginAttempt {
    pub count: u32,
    pub last_attempt: Instant,
}

pub struct AppState {
    pub docker: Docker,
    pub jwt_secret: String,
    pub login_attempts: DashMap<String, LoginAttempt>,
    pub user_repo: UserRepository,
    pub settings_repo: SettingsRepository,
    pub auth_required: AtomicBool,
}
