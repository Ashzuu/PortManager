use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json, Extension,
};
use bollard::{
    container::{
        ListContainersOptions, RemoveContainerOptions, RestartContainerOptions,
        StartContainerOptions, StopContainerOptions,
    },
    image::ListImagesOptions,
};
use std::sync::Arc;
use bollard::container::KillContainerOptions;
use bollard::models::PortTypeEnum;
use crate::{
    models::{
        auth::Claims,
        container::{Container, ContainerAction, DockerImage, PortMapping},
    },
    AppState,
};


pub async fn list_containers(
    State(state): State<Arc<AppState>>
) -> Result<Json<Vec<Container>>, StatusCode> {
    let mut options = ListContainersOptions::<String>::default();
    options.all = true;

    let d_containers = state.docker.list_containers(Some(options)).await.map_err(|e| {
        tracing::error!("Erreur lors de la récupération des conteneurs: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut result = Vec::new();

    for c in d_containers {
        let name = c.names.unwrap_or_default().first().unwrap_or(&"".to_string()).replace("/", "");

        let mut ports = Vec::new();
        if let Some(c_ports) = c.ports {
            for p in c_ports {
                ports.push(PortMapping {
                    private_port: p.private_port,
                    public_port: p.public_port,
                    r#type: p.typ.unwrap_or_else(|| PortTypeEnum::TCP).to_string(),
                });
            }
        }

        result.push(Container {
            id: c.id.unwrap_or_default().chars().take(12).collect(),
            name,
            image: c.image.unwrap_or_default(),
            status: c.status.unwrap_or_default(),
            state: c.state.unwrap_or_default(),
            ports,
            cpu_usage: "N/A".to_string(),
            memory_usage: "N/A".to_string(),
        });
    }

    Ok(Json(result))
}

pub async fn manage_container(
    State(state): State<Arc<AppState>>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(payload): Json<ContainerAction>,
) -> Result<impl IntoResponse, StatusCode> {

    let action_str = payload.action.as_str();
    tracing::info!("L'utilisateur '{}' demande l'action '{}' sur le conteneur '{}'", claims.sub, action_str, id);

    let result = match action_str {
        "start" => state.docker.start_container(&id, None::<StartContainerOptions<String>>).await,
        "stop" => state.docker.stop_container(&id, None::<StopContainerOptions>).await,
        "restart" => state.docker.restart_container(&id, None::<RestartContainerOptions>).await,
        "kill" => state.docker.kill_container(&id, None::<KillContainerOptions<String>>).await,
        "remove" => state.docker.remove_container(&id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    match result {
        Ok(_) => {
            tracing::info!("Action '{}' réussie sur '{}'", action_str, id);
            Ok(StatusCode::OK)
        }
        Err(e) => {
            tracing::error!("Échec de l'action '{}' sur '{}': {}", action_str, id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_images(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DockerImage>>, StatusCode> {

    let options = ListImagesOptions::<String> {
        all: true,
        ..Default::default()
    };

    let d_images = state.docker.list_images(Some(options)).await.map_err(|e| {
        tracing::error!("Erreur lors de la récupération des images: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let result = d_images.into_iter().map(|img| DockerImage {
        id: img.id.chars().skip(7).take(12).collect(),
        tags: img.repo_tags,
        size: img.size,
        created: img.created,
    }).collect();

    Ok(Json(result))
}