use axum::{response::IntoResponse, BoxError, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::out_time;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("{0}")]
    #[serde(with = "super::anyhow")]
    AnyhowError(anyhow::Error),

    #[error("request timeout {:?}", .0)]
    TimeoutError(std::time::Duration),
}

impl From<anyhow::Error> for ApiError {
    fn from(inner: anyhow::Error) -> Self {
        Self::AnyhowError(inner)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status_code, response) = match self {
            Self::TimeoutError(t) => (
                StatusCode::REQUEST_TIMEOUT,
                json!({ "msg": format!("{}", self) , "timeout": t.as_millis()}),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!({ "msg": format!("{}", self) }),
            ),
        };
        (status_code, Json(json!({ "error": response }))).into_response()
    }
}

impl ApiError {
    pub async fn handle(error: BoxError) -> (StatusCode, impl IntoResponse) {
        if error.is::<tower::timeout::error::Elapsed>() {
            (
                StatusCode::REQUEST_TIMEOUT,
                ApiError::TimeoutError(*out_time().await),
            )
        } else {
            let err = anyhow::anyhow!("Unhandled internal error: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ApiError::AnyhowError(err),
            )
        }
    }
}
