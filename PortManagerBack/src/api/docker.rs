use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json, Extension,
};
use std::sync::Arc;
use crate::{
    models::{
        auth::Claims,
        container::{Container, ContainerAction, DockerImage},
    },
    AppState,
};
use crate::services::docker_service::DockerService;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct DockerError {
    pub message: String,
}

impl IntoResponse for DockerError {
    fn into_response(self) -> Response {
        (StatusCode::SERVICE_UNAVAILABLE, Json(self)).into_response()
    }
}

#[utoipa::path(
    get,
    path = "/api/docker/containers",
    responses(
        (status = 200, description = "Liste des conteneurs récupérée avec succès", body = [Container]),
        (status = 401, description = "Non autorisé"),
        (status = 503, description = "Moteur Docker inaccessible", body = DockerError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_containers(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<Container>>, (StatusCode, Json<DockerError>)> {
    let containers = DockerService::list_containers(&state.docker).await?;
    Ok(Json(containers))
}

#[utoipa::path(
    post,
    path = "/api/docker/containers/{id}/action",
    params(
        ("id" = String, Path, description = "ID du conteneur")
    ),
    request_body = ContainerAction,
    responses(
        (status = 200, description = "Action effectuée avec succès"),
        (status = 400, description = "Action invalide"),
        (status = 401, description = "Non autorisé"),
        (status = 503, description = "Moteur Docker inaccessible", body = DockerError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn manage_container(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ContainerAction>,
) -> Result<impl IntoResponse, (StatusCode, Json<DockerError>)> {
    let action_str: &str = payload.action.as_str();
    tracing::info!("L'utilisateur '{}' demande l'action '{}' sur le conteneur '{}'", claims.sub, action_str, id);

    DockerService::manage_container(&state.docker, &id, action_str).await?;

    tracing::info!("Action '{}' réussie sur '{}'", action_str, id);
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/api/docker/images",
    responses(
        (status = 200, description = "Liste des images récupérée avec succès", body = [DockerImage]),
        (status = 401, description = "Non autorisé"),
        (status = 503, description = "Moteur Docker inaccessible", body = DockerError)
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_images(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DockerImage>>, (StatusCode, Json<DockerError>)> {
    let images = DockerService::list_images(&state.docker).await?;
    Ok(Json(images))
}
