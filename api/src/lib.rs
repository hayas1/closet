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

pub struct Configuration {}
impl Configuration {
    pub const HOST: (&str, &str) = ("HOST", "0.0.0.0");
    pub const PORT: (&str, &str) = ("PORT", "3000");
    pub const BASE_URL: (&str, &str) = ("BASE_URL", "/");
    pub const TIMEOUT: (&str, &str) = ("TIMEOUT", "1000ms");

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
            std::env::var(Self::BASE_URL.0)
                .unwrap_or_else(|_| Self::BASE_URL.1.into())
                .clone()
        })
    }

    pub fn out_time() -> &'static std::time::Duration {
        static _TIMEOUT: std::sync::OnceLock<std::time::Duration> = std::sync::OnceLock::new();
        _TIMEOUT.get_or_init(|| {
            let ts = std::env::var(Self::TIMEOUT.0).unwrap_or_else(|_| Self::TIMEOUT.1.into());
            duration_str::parse(&ts).unwrap_or_else(|e| panic!("{:?}", e))
        })
    }
}
