pub async fn api_route() -> axum::Router {
    axum::Router::new().route("/", axum::routing::get(|| async { "ok" }))
}

pub static LISTEN_DOMAIN: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();
pub async fn address() -> &'static str {
    LISTEN_DOMAIN
        .get_or_init(|| async {
            format!(
                "{}:{}",
                std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
                std::env::var("PORT").unwrap_or_else(|_| "3000".into()),
            )
        })
        .await
}
