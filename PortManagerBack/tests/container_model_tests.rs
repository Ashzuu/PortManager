use PortManagerBack::models::container::{Container, PortMapping};
use std::collections::HashMap;

#[test]
fn test_container_serialization() {
    let mut labels = HashMap::new();
    labels.insert("version".to_string(), "1.0".to_string());

    let container = Container {
        id: "123456789012".to_string(),
        name: "test_container".to_string(),
        image: "nginx:latest".to_string(),
        image_id: "abcdef123456".to_string(),
        command: "nginx -g 'daemon off;'".to_string(),
        created: 1622548800,
        status: "Up 2 hours".to_string(),
        state: "running".to_string(),
        ports: vec![PortMapping {
            private_port: 80,
            public_port: Some(8080),
            r#type: "tcp".to_string(),
        }],
        size_rw: Some(1024),
        size_root_fs: Some(2048),
        labels,
        cpu_usage: "0.5%".to_string(),
        memory_usage: "128MB".to_string(),
    };

    let serialized = serde_json::to_string(&container).unwrap();
    let deserialized: Container = serde_json::to_string(&container).and_then(|s| serde_json::from_str(&s)).unwrap();

    assert_eq!(deserialized.id, container.id);
    assert_eq!(deserialized.name, container.name);
    assert_eq!(deserialized.image_id, container.image_id);
    assert_eq!(deserialized.created, container.created);
    assert_eq!(deserialized.labels.get("version").unwrap(), "1.0");
    assert!(serialized.contains("\"imageId\":\"abcdef123456\""));
    assert!(serialized.contains("\"sizeRw\":1024"));
}
