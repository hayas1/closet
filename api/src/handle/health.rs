use axum::Router;

use crate::response::{result::ApiResponse, ApiResult};

pub fn health_router() -> Router {
    axum::Router::new().route("/", axum::routing::get(health))
}

pub async fn health<'a>() -> ApiResult<&'a str> {
    Ok(ApiResponse::new("ok"))
}

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;
    use hyper::{body::to_bytes, StatusCode};

    use super::*;

    #[tokio::test]
    async fn test_health() {
        let health = health().await.unwrap();
        assert_eq!(health.result(), &"ok");
    }

    #[tokio::test]
    async fn test_health_response() {
        let response = health().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = to_bytes(response.into_body()).await.unwrap();
        let health: ApiResponse<&str> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health, ApiResponse::new("ok"));
    }
}
