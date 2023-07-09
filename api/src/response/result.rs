use axum::{response::IntoResponse, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    result: T,
}
impl<T> ApiResponse<T> {
    pub fn new(result: T) -> Self {
        Self { result }
    }
    pub fn result(&self) -> &T {
        &self.result
    }
}
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> axum::response::Response {
        let with_status_code = (StatusCode::OK, Json(json!(self)));
        with_status_code.into_response()
    }
}
