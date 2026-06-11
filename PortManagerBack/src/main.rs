use axum::{routing::{get, post}, Router, middleware};
use PortManagerBack::api::auth::{login, auth_middleware, create_user, toggle_auth, list_users};
use PortManagerBack::api::docker::{list_containers, manage_container, list_images};
use PortManagerBack::api::nginx::list_nginx_configs;
use PortManagerBack::services::init_service::InitService;
use PortManagerBack::AppState;

use std::sync::Arc;
use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        PortManagerBack::api::auth::login,
        PortManagerBack::api::auth::create_user,
        PortManagerBack::api::auth::list_users,
        PortManagerBack::api::auth::toggle_auth,
        PortManagerBack::api::docker::list_containers,
        PortManagerBack::api::docker::manage_container,
        PortManagerBack::api::docker::list_images,
        PortManagerBack::api::nginx::list_nginx_configs,
    ),
    components(
        schemas(
            PortManagerBack::models::auth::LoginRequest,
            PortManagerBack::models::auth::CreateUserRequest,
            PortManagerBack::models::auth::UserResponse,
            PortManagerBack::models::auth::AuthResponse,
            PortManagerBack::api::auth::ToggleAuthRequest,
            PortManagerBack::api::auth::AuthError,
            PortManagerBack::models::container::Container,
            PortManagerBack::models::container::PortMapping,
            PortManagerBack::models::container::ContainerAction,
            PortManagerBack::models::container::DockerImage,
            PortManagerBack::models::nginx_config::NginxConfig,
            PortManagerBack::api::nginx::ApiError,
            PortManagerBack::api::docker::DockerError,
        )
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  tracing_subscriber::fmt::init();
  tracing::info!("Démarrage de l'agent");

  let shared_state: Arc<AppState> = InitService::init_app_state()?;

  let app: Router = create_router(shared_state);

  let listener: TcpListener = TcpListener::bind("localhost:8082").await?;
  tracing::info!("Agent backend listening on http://localhost:8082");

  axum::serve(listener, app).await?;
  Ok(())
}

fn create_router(state: Arc<AppState>) -> Router {
  let docker_routes: Router<Arc<AppState>> = Router::new()
        .route("/containers", get(list_containers))
        .route("/containers/:id/action", post(manage_container))
        .route("/images", get(list_images));

  let nginx_routes: Router<Arc<AppState>> = Router::new()
        .route("/", get(list_nginx_configs));

  let auth_routes: Router<Arc<AppState>> = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/toggle-auth", post(toggle_auth));

  let public_routes: Router<Arc<AppState>> = Router::new()
      .route("/api/health", get(health_check))
      .route("/api/login", post(login));

  let system_routes: Router<Arc<AppState>> = Router::new()
      .nest("/api/docker", docker_routes)
      .nest("/api/nginx", nginx_routes)
      .nest("/api/auth", auth_routes)
      .route_layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

  let mut router = Router::new()
      .merge(public_routes)
      .merge(system_routes);

  // Ajout de Swagger uniquement en développement
  let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
  if env == "development" {
      tracing::info!("Swagger UI disponible sur http://localhost:8082/swagger-ui");
      router = router.merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));
  }

  router.with_state(state)
}

async fn health_check() -> &'static str {
  "Agent Backend OK"
}
