use axum::{extract::Path, Router};
use hyper::StatusCode;
use serde::Serialize;
use serde_json::json;

use crate::{
    response::{result::ApiResponse, ApiResult},
    AppState,
};

pub fn dev_debug_router() -> Router<AppState> {
    axum::Router::new().route("/wait/:time", axum::routing::get(wait))
}

pub async fn wait(Path(duration): Path<String>) -> ApiResult<impl Serialize> {
    let d = duration_str::parse(&duration)
        .map_err(|e| (StatusCode::BAD_REQUEST, anyhow::anyhow!(e)))?;
    tokio::time::sleep(d).await;
    Ok(ApiResponse::Success(json!({ "waited": format!("{d:?}") })))
}
