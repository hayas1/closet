use axum::{extract::Path, Router};
use serde::Serialize;
use serde_json::json;

use crate::response::{result::ApiResponse, ApiResult};

pub fn dev_debug_router() -> Router {
    axum::Router::new().route("/wait/:time", axum::routing::get(wait))
}

#[tracing::instrument(level = "debug")]
pub async fn wait(Path(duration): Path<String>) -> ApiResult<impl Serialize> {
    let d = duration_str::parse(&duration).map_err(|e| anyhow::anyhow!(e))?;
    tokio::time::sleep(d).await;
    Ok(ApiResponse::new(json!({ "waited": format!("{d:?}") })))
}
