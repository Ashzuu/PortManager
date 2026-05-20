use axum::{routing::{get, post}, Router, middleware};
use crate::api::auth::{login, auth_middleware};
use crate::api::docker::{list_containers, manage_container, list_images};
use crate::services::docker_client::{init_docker_client};
use bollard::Docker;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

pub struct AppState {
  pub docker: Docker,
  pub jwt_secret: String,
}

pub mod api;
pub mod models;
pub mod services;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();
  tracing::info!("Démarrage de l'agent");

  let jwt_secret:String = std::env::var("JWT_SECRET").unwrap_or_else(|_| "secret_de_developpement".to_string());

  let docker_client:Docker = init_docker_client();

  let shared_state:Arc<AppState> = Arc::new(AppState {
    docker: docker_client,
    jwt_secret,
  });

  let docker_routes:Router<Arc<AppState>> = Router::new()
        .route("/containers", get(list_containers))
        .route("/containers/:id/action", post(manage_container))
        .route("/images", get(list_images));

  let public_routes:Router<Arc<AppState>> = Router::new()
      .route("/api/health", get(health_check))
      .route("/api/login", post(login));

  let system_routes:Router<Arc<AppState>> = Router::new()
      .nest("/api/docker", docker_routes)
      .route_layer(middleware::from_fn_with_state(shared_state.clone(), auth_middleware));

  let app = Router::new()
      .merge(public_routes)
      .merge(system_routes)
      .with_state(shared_state);

  let listener:TcpListener = TcpListener::bind("localhost:8082").await.unwrap();
  tracing::info!("Agent backend listening on http://localhost:8082");

  axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
  "Agent Backend OK"
}