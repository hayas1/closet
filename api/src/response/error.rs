use axum::{response::IntoResponse, BoxError, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr};

use crate::Configuration;

#[serde_as]
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("{1}")]
    #[serde(with = "self::serde_anyhow")]
    AnyhowError(StatusCode, #[source] anyhow::Error),

    #[error("{1}")] // TODO!!! should not response ???
    #[serde(with = "self::serde_database")]
    DatabaseError(StatusCode, #[source] sea_orm::DbErr),

    #[error("{1}")]
    EntityError(
        #[serde_as(as = "DisplayFromStr")] StatusCode,
        #[source] entity::error::validate::ValidateError,
    ),

    #[error("request timeout {:?}", .0)]
    TimeoutError(std::time::Duration),

    #[error("Not Found")]
    UnmatchedPathError,

    #[error("invalid username or password")]
    LoginFailError,
}
impl ApiError {
    pub fn status_code(&self) -> &StatusCode {
        match self {
            Self::AnyhowError(code, _) => code,
            Self::DatabaseError(code, _) => code,
            Self::EntityError(code, _) => code,
            Self::TimeoutError(_) => &StatusCode::REQUEST_TIMEOUT,
            Self::UnmatchedPathError => &StatusCode::NOT_FOUND,
            Self::LoginFailError => &StatusCode::FORBIDDEN,
        }
    }
    pub async fn handle_timeout(error: BoxError) -> impl IntoResponse {
        if error.is::<tower::timeout::error::Elapsed>() {
            ApiError::TimeoutError(*Configuration::out_time())
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
impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        Self::DatabaseError(StatusCode::INTERNAL_SERVER_ERROR, error)
    }
}
impl From<(StatusCode, sea_orm::DbErr)> for ApiError {
    fn from((status, error): (StatusCode, sea_orm::DbErr)) -> Self {
        Self::DatabaseError(status, error)
    }
}
impl From<entity::error::validate::ValidateError> for ApiError {
    fn from(error: entity::error::validate::ValidateError) -> Self {
        Self::EntityError(StatusCode::BAD_REQUEST, error)
    }
}
impl From<(StatusCode, entity::error::validate::ValidateError)> for ApiError {
    fn from((status, error): (StatusCode, entity::error::validate::ValidateError)) -> Self {
        Self::EntityError(status, error)
    }
}

mod serde_anyhow {
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(
        status: &StatusCode,
        anyhow_error: &anyhow::Error,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (status.as_u16(), anyhow_error.to_string()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(StatusCode, anyhow::Error), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (status, anyhow_msg) = <(u16, String)>::deserialize(deserializer)?;
        Ok((
            StatusCode::from_u16(status).expect("invalid status code"), // TODO error handling
            anyhow::Error::msg(anyhow_msg),
        ))
    }
}

// FIXME better serde implementation
mod serde_database {
    use hyper::StatusCode;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(
        status: &StatusCode,
        database_error: &sea_orm::DbErr,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (status.as_u16(), database_error.to_string()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(StatusCode, sea_orm::DbErr), D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (status, database_msg) = <(u16, String)>::deserialize(deserializer)?;
        Ok((
            StatusCode::from_u16(status).expect("invalid status code"),
            sea_orm::DbErr::Custom(database_msg),
        ))
    }
}
