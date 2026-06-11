use axum::{
    extract::{State, Request},
    http::{header, StatusCode, HeaderValue},
    middleware::Next,
    response::{Response, IntoResponse},
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation, TokenData};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use crate::{AppState, models::auth::{Claims, LoginRequest, AuthResponse, CreateUserRequest, UserResponse}};
use serde::{Serialize, Deserialize};
use crate::services::auth_service::AuthService;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct ToggleAuthRequest {
    pub required: bool,
}

#[derive(Serialize, ToSchema, Debug)]
pub struct AuthError {
    pub message: String,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after: Option<u64>,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let status: StatusCode = match self.code.as_str() {
            "TOO_MANY_REQUESTS" => StatusCode::TOO_MANY_REQUESTS,
            "INVALID_CREDENTIALS" => StatusCode::UNAUTHORIZED,
            "UNAUTHORIZED" => StatusCode::UNAUTHORIZED,
            "ALREADY_EXISTS" => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(self)).into_response()
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/toggle-auth",
    request_body = ToggleAuthRequest,
    responses(
        (status = 200, description = "Authentification basculée avec succès"),
        (status = 401, description = "Non autorisé"),
        (status = 500, description = "Erreur interne", body = AuthError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn toggle_auth(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ToggleAuthRequest>,
) -> Result<StatusCode, AuthError> {
    AuthService::toggle_auth(&state, payload.required).await?;
    tracing::info!("Authentification requise mise à jour: {}", payload.required);
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/auth/users",
    responses(
        (status = 200, description = "Liste des utilisateurs récupérée avec succès", body = [UserResponse]),
        (status = 401, description = "Non autorisé"),
        (status = 500, description = "Erreur interne", body = AuthError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponse>>, AuthError> {
    let users = AuthService::list_users(&state).await?;
    Ok(Json(users))
}

#[utoipa::path(
    post,
    path = "/api/auth/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "Utilisateur créé avec succès"),
        (status = 401, description = "Non autorisé"),
        (status = 409, description = "Utilisateur existe déjà", body = AuthError),
        (status = 500, description = "Erreur interne", body = AuthError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<StatusCode, AuthError> {
    AuthService::create_user(&state, payload).await?;
    tracing::info!("Nouvel utilisateur créé");
    Ok(StatusCode::CREATED)
}

#[utoipa::path(
    post,
    path = "/api/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Connexion réussie", body = AuthResponse),
        (status = 401, description = "Identifiants invalides", body = AuthError),
        (status = 429, description = "Trop de tentatives", body = AuthError),
        (status = 500, description = "Erreur interne", body = AuthError)
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AuthError> {
    match AuthService::login(&state, payload).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err(e)
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Si l'authentification n'est pas requise, on passe directement
    if !state.auth_required.load(Ordering::SeqCst) {
        return Ok(next.run(req).await);
    }

    let auth_header: Option<String> = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|val: &HeaderValue| val.to_str().ok())
        .filter(|val: &&str| val.starts_with("Bearer "))
        .map(|val: &str| val[7..].to_string());

    let token: String = match auth_header {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token_data: TokenData<Claims> = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(state.jwt_secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| {
        tracing::warn!("Tentative d'accès avec un token invalide ou expiré");
        StatusCode::UNAUTHORIZED
    })?;

    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}
