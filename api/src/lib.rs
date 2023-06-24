pub mod dev_debug;
pub mod handle;
pub mod middleware;
pub mod response;

pub fn router() -> axum::Router<AppState> {
    axum::Router::new().nest(Configuration::base_url(), api_router())
}
pub fn api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(handle::health::health))
        .nest("/dev/debug", dev_debug::dev_debug_router())
        .nest("/health", handle::health::health_router())
        .nest("/auth", handle::auth::auth_router())
        .route("/*404", axum::routing::get(response::error::ApiError::handle_not_found))
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::error_handling::HandleErrorLayer::new(
                    response::error::ApiError::handle_timeout,
                ))
                .timeout(*Configuration::out_time()),
        )
        .layer(axum::middleware::from_fn(middleware::logging::request_log))
}
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub encoding_key: jsonwebtoken::EncodingKey,
    pub decoding_key: jsonwebtoken::DecodingKey,
}
pub async fn with_database_connection(
    router: axum::Router<AppState>,
) -> Result<axum::Router, sea_orm::DbErr> {
    let db = sea_orm::Database::connect(Configuration::database_uri());
    let secret = Configuration::secret_key();
    Ok(router.with_state(AppState {
        db: db.await?,
        encoding_key: jsonwebtoken::EncodingKey::from_secret(secret.as_ref()),
        decoding_key: jsonwebtoken::DecodingKey::from_secret(secret.as_ref()),
    }))
}

pub struct Configuration {}
impl Configuration {
    pub const HOST: (&str, &str) = ("HOST", "0.0.0.0");
    pub const PORT: (&str, &str) = ("PORT", "3000");
    pub const BASE_URL: (&str, &str) = ("BASE_URL", "/");
    pub const TIMEOUT: (&str, &str) = ("TIMEOUT", "1000ms");
    pub const SECRET_KEY: &str = "SECRET_KEY";
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
            format!("{}:{}", ip, port).parse().unwrap_or_else(|e| panic!("{}", e))
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

    pub fn secret_key() -> &'static str {
        static _SECRET_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        _SECRET_KEY.get_or_init(|| {
            std::env::var(Self::SECRET_KEY)
                .unwrap_or_else(|e| panic!("{}: {}", e, Self::SECRET_KEY))
        })
    }

    pub fn database_uri() -> &'static str {
        static _DATABASE_URI: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        _DATABASE_URI.get_or_init(|| match std::env::var(Self::DATABASE_URL) {
            Ok(uri) => uri,
            Err(_) => format!(
                "mysql://{}:{}@{}:{}/{}",
                std::env::var(Self::MYSQL_USER).unwrap_or_else(|e| panic!(
                    "{}: {} or {}",
                    e,
                    Self::MYSQL_USER,
                    Self::DATABASE_URL,
                )),
                std::env::var(Self::MYSQL_PASSWORD).unwrap_or_else(|e| panic!(
                    "{}: {} or {}",
                    e,
                    Self::MYSQL_PASSWORD,
                    Self::DATABASE_URL,
                )),
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
    use jsonwebtoken::{DecodingKey, EncodingKey};
    use sea_orm::DatabaseConnection;
    use tower::Service;

    use crate::response::{message::Either, result::ApiResponse};

    use super::*;

    #[tokio::test]
    async fn test_health_call() {
        let (uri, body) = ("/health", Body::empty());
        let mut api = api_router()
            .with_state(AppState {
                db: DatabaseConnection::Disconnected,
                encoding_key: EncodingKey::from_secret("secret".as_ref()),
                decoding_key: DecodingKey::from_secret("secret".as_ref()),
            })
            .into_make_service();
        let request = Request::builder().uri(uri).body(body).unwrap();
        let mut router = api.call(&request).await.unwrap();
        let response = router.call(request).await.unwrap(); // request twice ?

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":"ok"}"#);
        let health: ApiResponse<Either> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health, ApiResponse::new(Either::Ok));
    }
}
