use axum::response::IntoResponse;

use crate::response::{result::ApiResponse, ApiResult};

#[tracing::instrument(level = "info")]
pub async fn health() -> ApiResult<impl IntoResponse> {
    Ok(ApiResponse::new("ok"))
}
