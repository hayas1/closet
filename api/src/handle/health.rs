use axum::{extract::State, Router};
use hyper::StatusCode;
use sea_orm::{EntityTrait, FromJsonQueryResult, FromQueryResult};
use serde::{Deserialize, Serialize};

use crate::{
    response::{result::ApiResponse, ApiResult},
    AppState,
};

#[derive(
    Debug, Clone, Eq, PartialEq, FromQueryResult, FromJsonQueryResult, Serialize, Deserialize,
)]
pub struct RichHealth {
    pub status: String,
}
pub fn health_router() -> Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(health))
        .route("/rich", axum::routing::get(rich_health))
}

pub async fn health<'a>() -> ApiResult<&'a str> {
    Ok(ApiResponse::new("ok"))
}
pub async fn rich_health(State(state): State<AppState>) -> ApiResult<RichHealth> {
    let health = entity::health::Entity::find()
        .into_model::<RichHealth>()
        .one(&state.db)
        .await?
        .ok_or_else(|| {
            (StatusCode::INTERNAL_SERVER_ERROR, anyhow::anyhow!("no record in database"))
        })?;
    Ok(ApiResponse::new(health))
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

    // FIXME 2023/06/24 when DatabaseConnection::Disconnected, query of sea_orm 0.11.3 will panic, not return Err
    // #[tokio::test]
    // async fn test_rich_health_without_db() {
    //     let health = rich_health(State(AppState {
    //         db: sea_orm::DatabaseConnection::Disconnected,
    //     }))
    //     .await;
    //     let err = health.unwrap_err();
    //     assert!(matches!(err, ApiError::DatabaseError(_, _)));
    // }
}
