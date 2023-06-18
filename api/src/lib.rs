pub mod handle;
pub mod response;

pub const HOST: (&str, &str) = ("HOST", "0.0.0.0");
pub const PORT: (&str, &str) = ("PORT", "3000");
pub const BASE_URL: (&str, &str) = ("BASE_URL", "/");

pub async fn router() -> axum::Router {
    axum::Router::new().nest(base_url().await, api_router())
}
pub fn api_router() -> axum::Router {
    axum::Router::new()
        .route("/", axum::routing::get(handle::health::health))
        .nest("/health", handle::health::health_router())
}

pub static _ADDRESS: tokio::sync::OnceCell<std::net::SocketAddr> =
    tokio::sync::OnceCell::const_new();
pub async fn address() -> &'static std::net::SocketAddr {
    _ADDRESS
        .get_or_init(|| async {
            let (ip, port) = (
                std::env::var(HOST.0).unwrap_or_else(|_| HOST.1.into()),
                std::env::var(PORT.0).unwrap_or_else(|_| PORT.1.into()),
            );
            format!("{}:{}", ip, port)
                .parse()
                .unwrap_or_else(|e| panic!("{}", e))
        })
        .await
}

pub static _BASE_URL: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();
pub async fn base_url() -> &'static str {
    _BASE_URL
        .get_or_init(|| async { std::env::var(BASE_URL.0).unwrap_or_else(|_| BASE_URL.1.into()) })
        .await
}
