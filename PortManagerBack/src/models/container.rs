use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct PortMapping {
    #[serde(rename = "privatePort")]
    pub private_port: u16,
    #[serde(rename = "publicPort")]
    pub public_port: Option<u16>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    #[serde(rename = "imageId")]
    pub image_id: String,
    pub command: String,
    pub created: i64,
    pub status: String,
    pub state: String,
    pub ports: Vec<PortMapping>,
    #[serde(rename = "sizeRw")]
    pub size_rw: Option<i64>,
    #[serde(rename = "sizeRootFs")]
    pub size_root_fs: Option<i64>,
    pub labels: std::collections::HashMap<String, String>,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: String,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: String,
}

#[derive(Deserialize, ToSchema)]
pub struct ContainerAction {
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct DockerImage {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}
