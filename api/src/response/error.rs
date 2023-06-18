use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("{0}")]
    #[serde(with = "super::anyhow")]
    AnyhowMsgError(anyhow::Error),
}

impl From<anyhow::Error> for ApiError {
    fn from(inner: anyhow::Error) -> Self {
        Self::AnyhowMsgError(inner)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let with_status_code = (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("{}", self) })).into_response(),
        );
        with_status_code.into_response()
    }
}
