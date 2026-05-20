use axum::{
    extract::{State, Request},
    http::{header, StatusCode},
    middleware::Next,
    response::{Response},
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::{sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use crate::{AppState, models::auth::{Claims, LoginRequest, AuthResponse}};

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {

    // TODO: Connecter ici la logique anti-brute force (fail2ban/rate limiter mémoire)

    if payload.username != "admin" || payload.password != "password" {
        tracing::warn!("Tentative de connexion échouée pour l'utilisateur: {}", payload.username);
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expiration:usize = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + (2 * 3600);

    let claims = Claims {
        sub: payload.username.clone(),
        exp: expiration,
    };

    let token:String = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(state.jwt_secret.as_bytes()),
    ).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    tracing::info!("Connexion réussie pour l'utilisateur: {}", payload.username);
    Ok(Json(AuthResponse { token }))
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    let auth_header = req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .filter(|val| val.starts_with("Bearer "))
        .map(|val| val[7..].to_string());

    let token = match auth_header {
        Some(t) => t,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let token_data = decode::<Claims>(
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