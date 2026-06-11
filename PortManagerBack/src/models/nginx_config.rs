use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NginxConfig {
    pub filename: String,
    pub domain: Option<String>,
    pub target_proxy: Option<String>,
    pub ssl_enabled: bool,
    pub is_active: bool,
    pub raw_content: String,
}
