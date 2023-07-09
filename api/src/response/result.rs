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
    pub fn result(&self) -> &T {
        match self {
            Self::Success(ok) => ok,
            Self::Failure(err) => panic!("{:?}", err),
        }
    }
    pub fn unwrap_err(&self) -> &ApiError {
        match self {
            Self::Success(_) => panic!("unexpected success"),
            Self::Failure(err) => err,
        }
    }
}
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let with_status_code = (StatusCode::OK, Json(json!(self)));
        with_status_code.into_response()
    }
}
