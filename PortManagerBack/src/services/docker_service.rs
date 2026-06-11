use bollard::Docker;
use bollard::container::{ListContainersOptions, StartContainerOptions, StopContainerOptions, RestartContainerOptions, KillContainerOptions, RemoveContainerOptions};
use bollard::image::ListImagesOptions;
use crate::models::container::{Container, DockerImage, PortMapping};
use bollard::models::PortTypeEnum;
use crate::api::docker::DockerError;
use axum::http::StatusCode;
use axum::Json;

pub struct DockerService;

impl DockerService {
    pub async fn list_containers(docker: &Docker) -> Result<Vec<Container>, (StatusCode, Json<DockerError>)> {
        let mut options: ListContainersOptions<String> = ListContainersOptions::<String>::default();
        options.all = true;
        options.size = true;

        let d_containers = docker.list_containers(Some(options)).await.map_err(Self::handle_docker_error)?;

        let mut result: Vec<Container> = Vec::new();

        for c in d_containers {
            let name: String = c.names.unwrap_or_default().first().unwrap_or(&"".to_string()).replace("/", "");

            let mut ports: Vec<PortMapping> = Vec::new();
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
                image_id: c.image_id.unwrap_or_default().chars().skip(7).take(12).collect(),
                command: c.command.unwrap_or_default(),
                created: c.created.unwrap_or(0),
                status: c.status.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                ports,
                size_rw: c.size_rw,
                size_root_fs: c.size_root_fs,
                labels: c.labels.unwrap_or_default(),
                cpu_usage: "N/A".to_string(),
                memory_usage: "N/A".to_string(),
            });
        }

        Ok(result)
    }

    pub async fn manage_container(docker: &Docker, id: &str, action: &str) -> Result<(), (StatusCode, Json<DockerError>)> {
        let result = match action {
            "start" => docker.start_container(id, None::<StartContainerOptions<String>>).await,
            "stop" => docker.stop_container(id, None::<StopContainerOptions>).await,
            "restart" => docker.restart_container(id, None::<RestartContainerOptions>).await,
            "kill" => docker.kill_container(id, None::<KillContainerOptions<String>>).await,
            "remove" => docker.remove_container(id, Some(RemoveContainerOptions { force: true, ..Default::default() })).await,
            _ => return Err((StatusCode::BAD_REQUEST, Json(DockerError { message: "Action invalide.".to_string() }))),
        };

        result.map_err(Self::handle_docker_error)
    }

    pub async fn list_images(docker: &Docker) -> Result<Vec<DockerImage>, (StatusCode, Json<DockerError>)> {
        let options: ListImagesOptions<String> = ListImagesOptions::<String> {
            all: true,
            ..Default::default()
        };

        let d_images = docker.list_images(Some(options)).await.map_err(Self::handle_docker_error)?;

        let result: Vec<DockerImage> = d_images.into_iter().map(|img| DockerImage {
            id: img.id.chars().skip(7).take(12).collect(),
            tags: img.repo_tags,
            size: img.size,
            created: img.created,
        }).collect();

        Ok(result)
    }

    fn handle_docker_error(e: bollard::errors::Error) -> (StatusCode, Json<DockerError>) {
        tracing::error!("Erreur Docker: {}", e);
        
        let message: String = match e {
            bollard::errors::Error::DockerResponseServerError { status_code, .. } if status_code == 404 => {
                "Ressource Docker non trouvée.".to_string()
            },
            _ => {
                "Impossible de communiquer avec le moteur Docker. Vérifiez que le service Docker est lancé et que l'agent a les permissions nécessaires (socket).".to_string()
            }
        };

        (StatusCode::SERVICE_UNAVAILABLE, Json(DockerError { message }))
    }
}
