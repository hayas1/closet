use axum::response::IntoResponse;
use tokio::time::Instant;

pub async fn request_log<B>(
    req: hyper::Request<B>,
    next: axum::middleware::Next<B>,
) -> impl IntoResponse {
    let (method, uri) = (req.method().clone(), req.uri().clone());

    let start = Instant::now();
    let res = next.run(req).await;
    let latency = start.elapsed();

    // TODO better log
    tracing::info!("{} {} {} {:?}", method, uri, res.status(), latency);
    res
}
