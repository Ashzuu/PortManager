use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, ToSchema, Clone, Debug)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, ToSchema, Clone, Debug)]
pub struct AuthResponse {
    pub token: String,
}

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}
