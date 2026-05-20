use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PortMapping {
    #[serde(rename = "privatePort")]
    pub private_port: u16,
    #[serde(rename = "publicPort")]
    pub public_port: Option<u16>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub ports: Vec<PortMapping>,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: String,
    #[serde(rename = "memoryUsage")]
    pub memory_usage: String,
}

#[derive(Deserialize)]
pub struct ContainerAction {
    pub action: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DockerImage {
    pub id: String,
    pub tags: Vec<String>,
    pub size: i64,
    pub created: i64,
}