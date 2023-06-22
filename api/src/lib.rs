pub mod dev_debug;
pub mod handle;
pub mod logging;
pub mod response;

pub fn router() -> axum::Router {
    axum::Router::new().nest(Configuration::base_url(), api_router())
}
pub fn api_router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(handle::health::health))
        .nest("/dev/debug", dev_debug::dev_debug_router())
        .nest("/health", handle::health::health_router())
        .route(
            "/*404",
            axum::routing::get(response::error::ApiError::handle_not_found),
        )
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::error_handling::HandleErrorLayer::new(
                    response::error::ApiError::handle_timeout,
                ))
                .timeout(*Configuration::out_time()),
        )
        .layer(axum::middleware::from_fn(logging::middleware::request_log))
}
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
}
pub async fn with_database_connection(
    router: axum::Router<AppState>,
) -> Result<axum::Router<AppState>, sea_orm::DbErr> {
    let db = sea_orm::Database::connect(Configuration::database_uri());
    Ok(router.with_state(AppState { db: db.await? }))
}

pub struct Configuration {}
impl Configuration {
    pub const HOST: (&str, &str) = ("HOST", "0.0.0.0");
    pub const PORT: (&str, &str) = ("PORT", "3000");
    pub const BASE_URL: (&str, &str) = ("BASE_URL", "/");
    pub const TIMEOUT: (&str, &str) = ("TIMEOUT", "1000ms");
    pub const DATABASE_URL: &str = "DATABASE_URL"; // sea_orm require env DATABASE_URL
    pub const MYSQL_HOST: (&str, &str) = ("MYSQL_HOST", "127.0.0.1");
    pub const MYSQL_USER: &str = "MYSQL_USER";
    pub const MYSQL_PASSWORD: &str = "MYSQL_PASSWORD";
    pub const MYSQL_PORT: (&str, &str) = ("MYSQL_PORT", "3306");
    pub const MYSQL_DB: (&str, &str) = ("MYSQL_DB", "db");

    pub fn address() -> &'static std::net::SocketAddr {
        static _ADDRESS: std::sync::OnceLock<std::net::SocketAddr> = std::sync::OnceLock::new();
        _ADDRESS.get_or_init(|| {
            let (ip, port) = (
                std::env::var(Self::HOST.0).unwrap_or_else(|_| Self::HOST.1.into()),
                std::env::var(Self::PORT.0).unwrap_or_else(|_| Self::PORT.1.into()),
            );
            format!("{}:{}", ip, port)
                .parse()
                .unwrap_or_else(|e| panic!("{}", e))
        })
    }

    pub fn base_url() -> &'static str {
        static _BASE_URL: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        _BASE_URL.get_or_init(|| {
            std::env::var(Self::BASE_URL.0).unwrap_or_else(|_| Self::BASE_URL.1.into())
        })
    }

    pub fn out_time() -> &'static std::time::Duration {
        static _TIMEOUT: std::sync::OnceLock<std::time::Duration> = std::sync::OnceLock::new();
        _TIMEOUT.get_or_init(|| {
            let ts = std::env::var(Self::TIMEOUT.0).unwrap_or_else(|_| Self::TIMEOUT.1.into());
            duration_str::parse(&ts).unwrap_or_else(|e| panic!("{:?}", e))
        })
    }

    pub fn database_uri() -> &'static str {
        static _DATABASE_URI: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        _DATABASE_URI.get_or_init(|| match std::env::var(Self::DATABASE_URL) {
            Ok(uri) => uri,
            Err(_) => format!(
                "mysql://{}:{}@{}:{}/{}",
                std::env::var(Self::MYSQL_USER).unwrap(),
                std::env::var(Self::MYSQL_PASSWORD).unwrap(),
                std::env::var(Self::MYSQL_HOST.0).unwrap_or_else(|_| Self::MYSQL_HOST.1.into()),
                std::env::var(Self::MYSQL_PORT.0).unwrap_or_else(|_| Self::MYSQL_PORT.1.into()),
                std::env::var(Self::MYSQL_DB.0).unwrap_or_else(|_| Self::MYSQL_DB.1.into()),
            ),
        })
    }
}

#[cfg(test)]
mod tests {
    use hyper::{body::to_bytes, Body, Request, StatusCode};
    use tower::Service;

    use crate::response::result::ApiResponse;

    use super::*;

    #[tokio::test]
    async fn test_health_call() {
        let (uri, body) = ("/health", Body::empty());
        let mut api = api_router().into_make_service();
        let request = Request::builder().uri(uri).body(body).unwrap();
        let mut router = api.call(&request).await.unwrap();
        let response = router.call(request).await.unwrap(); // request twice ?

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":"ok"}"#);
        let health: ApiResponse<&str> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health, ApiResponse::new("ok"));
    }
}
