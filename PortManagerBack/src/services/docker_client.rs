use bollard::Docker;

pub fn init_docker_client() -> Docker {
    match Docker::connect_with_socket_defaults(){
        Ok(client) => {
            tracing::info!("Docker client initialized successfully with UNIX system");
            return client
        }
        Err(_) => {
            match Docker::connect_with_local_defaults(){
                Ok(client) => {
                    tracing::info!("Docker client initialized successfully with local system");
                    return client
                }
                Err(_) => {
                    tracing::error!("Docker client could not connect to docker daemon");
                    panic!("Failed to initialize Docker client with both UNIX and local defaults");
                }
            }
        }
    }
}