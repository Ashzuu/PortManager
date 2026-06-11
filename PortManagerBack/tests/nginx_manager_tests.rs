use PortManagerBack::services::nginx_service::*;

#[test]
fn test_extract_domain() {
    let content = "
        server {
            listen 80;
            server_name example.com;
            location / {
                proxy_pass http://localhost:3000;
            }
        }
    ";
    assert_eq!(extract_domain(content), Some("example.com".to_string()));
}

#[test]
fn test_extract_domain_multiple_whitespace() {
    let content = "  server_name   my-app.local;  ";
    assert_eq!(extract_domain(content), Some("my-app.local".to_string()));
}

#[test]
fn test_extract_domain_none() {
    let content = "listen 80;";
    assert_eq!(extract_domain(content), None);
}

#[test]
fn test_extract_proxy() {
    let content = "
        location / {
            proxy_pass http://192.168.1.10:8080;
        }
    ";
    assert_eq!(extract_proxy(content), Some("http://192.168.1.10:8080".to_string()));
}

#[test]
fn test_extract_proxy_none() {
    let content = "server_name example.com;";
    assert_eq!(extract_proxy(content), None);
}
