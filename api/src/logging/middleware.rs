use axum::response::IntoResponse;

pub async fn request_log<B>(
    req: hyper::Request<B>,
    next: axum::middleware::Next<B>,
) -> impl IntoResponse {
    let (method, uri) = (req.method().clone(), req.uri().clone());
    let res = next.run(req).await;
    // TODO better log
    tracing::info!("{} {} {}", method, uri, res.status());
    res
}
