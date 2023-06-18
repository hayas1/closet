use axum::{response::IntoResponse, Router};

use crate::response::{result::ApiResponse, ApiResult};

pub fn health_router() -> Router {
    axum::Router::new().route("/", axum::routing::get(health))
}

#[tracing::instrument(level = "info")]
pub async fn health() -> ApiResult<impl IntoResponse> {
    Ok(ApiResponse::new("ok"))
}
