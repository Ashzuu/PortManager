use axum::{Json, response::IntoResponse, http::StatusCode};
use crate::services::nginx_service::NginxService;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/api/nginx",
    responses(
        (status = 200, description = "Liste des configurations Nginx récupérée avec succès", body = [NginxConfig]),
        (status = 401, description = "Non autorisé"),
        (status = 500, description = "Erreur interne", body = ApiError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_nginx_configs() -> impl IntoResponse {
    match NginxService::list_active_configs() {
        Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
        Err(e) => {
             tracing::error!("Erreur Nginx: {}", e);
            
            let error_response = ApiError {
                message: "Une erreur interne est survenue lors de la récupération des configurations Nginx.".to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        },
    }
}
