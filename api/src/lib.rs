use configuration::Configuration;

pub mod configuration;
pub mod dev_debug;
pub mod handler;
pub mod middleware;
pub mod response;

pub fn router(base_url: &str) -> axum::Router<AppState> {
    axum::Router::new().nest(base_url, api_router())
}
pub fn api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(handler::health::health))
        .nest("/dev/debug", dev_debug::dev_debug_router())
        .nest("/health", handler::health::health_router())
        .nest("/auth", handler::auth::auth_router())
        .route("/*404", axum::routing::any(response::error::ApiError::handle_not_found))
}
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub configuration: configuration::Configuration,
}
pub async fn with_auth(
    router: axum::Router<AppState>,
    configuration: Configuration,
) -> Result<axum::Router, sea_orm::DbErr> {
    let db = sea_orm::Database::connect(configuration.database_url()).await?;
    if configuration.migrate() {
        use migration::{Migrator, MigratorTrait};
        Migrator::up(&db, None).await?;
    }
    let state = AppState { db, configuration };
    let timeout = state.clone().configuration.timeout().to_std().unwrap(); // TODO error handling
    Ok(router
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(state, middleware::authorization::verification))
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::error_handling::HandleErrorLayer::new(
                    response::error::ApiError::handle_timeout,
                ))
                .timeout(timeout),
        )
        .layer(axum::middleware::from_fn(middleware::logging::request_log)))
}

#[cfg(all(test, feature = "sqlite"))]
pub async fn standalone() -> Result<axum::Router, sea_orm::DbErr> {
    use rand::distributions::{Alphanumeric, DistString};

    let configuration = configuration::Configuration::new(configuration::Config {
        database_url: Some("sqlite::memory:".into()),
        secret_key: Some(Alphanumeric.sample_string(&mut rand::thread_rng(), 1024)),
        migrate: Some(true),
        ..Default::default()
    });
    with_auth(router(configuration.base_url()), configuration).await
}

#[cfg(test)]
mod tests {
    use entity::class::status::Status;
    use hyper::{body::to_bytes, Body, Request, StatusCode};
    use sea_orm::DatabaseConnection;
    use tower::Service;

    use crate::response::result::ApiResponse;

    use super::*;

    #[tokio::test]
    async fn test_health_call() {
        let (uri, body) = ("/health", Body::empty());
        let mut api = api_router()
            .with_state(AppState {
                db: DatabaseConnection::Disconnected,
                configuration: Configuration::new(Default::default()),
            })
            .into_make_service();
        let request = Request::builder().uri(uri).body(body).unwrap();
        let mut router = api.call(&request).await.unwrap();
        let response = router.call(request).await.unwrap(); // request twice ?

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":"ok"}"#);
        let health: ApiResponse<Status> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health, ApiResponse::new(Status::Ok));
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_rich_health_call() {
        use crate::handler::health::RichHealth;

        let (uri, body) = ("/health/rich", Body::empty());
        let mut api = standalone().await.unwrap().into_make_service();
        let request = Request::builder().uri(uri).body(body).unwrap();
        let mut router = api.call(&request).await.unwrap();
        let response = router.call(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":{"status":"ok"}}"#);
        let health: ApiResponse<RichHealth> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health, ApiResponse::new(RichHealth { status: Status::Ok }));
    }
}
