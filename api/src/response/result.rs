use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{error::ApiError, ApiResult};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ApiResponse<T> {
    Success(T),
    Failure(ApiError),
}
impl<T> ApiResponse<T> {
    pub fn new(result: T) -> Self {
        Self::Success(result)
    }
    pub fn failure(err: ApiError) -> Self {
        Self::Failure(err)
    }
    pub fn result(&self) -> Result<&T, &ApiError> {
        match self {
            Self::Success(ok) => Ok(ok),
            Self::Failure(err) => Err(err),
        }
    }
}
impl<T> Into<ApiResult<T>> for ApiResponse<T> {
    fn into(self) -> ApiResult<T> {
        match self {
            Self::Success(ok) => Ok(Self::new(ok)),
            Self::Failure(err) => Err(err),
        }
    }
}
impl<T> Into<Result<T, ApiError>> for ApiResponse<T> {
    fn into(self) -> Result<T, ApiError> {
        match self {
            Self::Success(ok) => Ok(ok),
            Self::Failure(err) => Err(err),
        }
    }
}
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let with_status_code = (StatusCode::OK, Json(json!(self)));
        with_status_code.into_response()
    }
}
