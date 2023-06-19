use axum::{response::IntoResponse, BoxError, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::out_time;

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("{1}")]
    #[serde(with = "super::anyhow")]
    AnyhowError(StatusCode, anyhow::Error),

    #[error("request timeout {:?}", .0)]
    TimeoutError(std::time::Duration),

    #[error("Not Found")]
    UnmatchedPathError,
}
impl ApiError {
    pub fn status_code(&self) -> &StatusCode {
        match self {
            Self::AnyhowError(code, _) => &code,
            Self::TimeoutError(_) => &StatusCode::REQUEST_TIMEOUT,
            Self::UnmatchedPathError => &StatusCode::NOT_FOUND,
        }
    }
    pub async fn handle_timeout(error: BoxError) -> impl IntoResponse {
        if error.is::<tower::timeout::error::Elapsed>() {
            ApiError::TimeoutError(*out_time().await)
        } else {
            let err = anyhow::anyhow!("Unhandled internal error: {}", error);
            err.into()
        }
    }
    pub async fn handle_not_found() -> impl IntoResponse {
        Self::UnmatchedPathError
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let error = json!({"msg": format!("{}", self), "serde": self});
        (self.status_code().clone(), Json(json!({ "error": error }))).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self::AnyhowError(StatusCode::INTERNAL_SERVER_ERROR, error)
    }
}
impl From<(StatusCode, anyhow::Error)> for ApiError {
    fn from((status, error): (StatusCode, anyhow::Error)) -> Self {
        Self::AnyhowError(status, error)
    }
}
