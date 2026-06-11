use std::fs;
use std::path::Path;
use crate::models::nginx_config::NginxConfig;

pub struct NginxService;

impl NginxService {
    pub fn list_active_configs() -> Result<Vec<NginxConfig>, String> {
        let sites_enabled_path: &str = "/etc/nginx/sites-enabled/";
        let path: &Path = Path::new(sites_enabled_path);

        if !path.exists() {
            return Ok(Vec::<NginxConfig>::new()); // Nginx non installé ou pas de sites activés
        }

        let mut configs: Vec<NginxConfig> = Vec::new();

        let entries: fs::ReadDir = fs::read_dir(path).map_err(|e: std::io::Error| format!("Erreur lors de la lecture de {}: {}", sites_enabled_path, e))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path: std::path::PathBuf = entry.path();
                if path.is_file() || path.is_symlink() {
                    let filename: String = entry.file_name().into_string().unwrap_or_default();
                    let content: String = fs::read_to_string(&path).unwrap_or_default();

                    // Analyse basique du contenu pour extraire le domaine et le proxy
                    let domain: Option<String> = extract_domain(&content);
                    let target_proxy: Option<String> = extract_proxy(&content);
                    let ssl_enabled: bool = content.contains("listen 443") || content.contains("ssl on");

                    configs.push(NginxConfig {
                        filename,
                        domain,
                        target_proxy,
                        ssl_enabled,
                        is_active: true,
                        raw_content: content,
                    });
                }
            }
        }

        Ok(configs)
    }
}

pub fn extract_domain(content: &str) -> Option<String> {
    for line in content.lines() {
        let line: &str = line.trim();
        if line.starts_with("server_name") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                return Some(parts[1].trim_matches(';').to_string());
            }
        }
    }
    None
}

pub fn extract_proxy(content: &str) -> Option<String> {
    for line in content.lines() {
        let line: &str = line.trim();
        if line.starts_with("proxy_pass") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                return Some(parts[1].trim_matches(';').to_string());
            }
        }
    }
    None
}
