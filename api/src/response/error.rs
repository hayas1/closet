use axum::{response::IntoResponse, BoxError, Json};
use chrono::Duration;
use entity::error::EntityError;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr};

use crate::configuration::Config;

use super::result::ApiResponse;

#[serde_as]
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ApiError {
    #[error("{}", source)]
    AnyhowError {
        #[serde_as(as = "DisplayFromStr")]
        code: StatusCode,
        #[source]
        #[serde(with = "self::serde_anyhow")]
        source: anyhow::Error,
    },

    #[error("cannot connect database")]
    CannotConnectDatabase,
    #[error("record not found")]
    RecordNotFound,
    #[error("unexpected database error")]
    UnexpectedDatabaseError {
        #[serde_as(as = "DisplayFromStr")]
        code: StatusCode,
    },

    #[error("{}", source)]
    EntityError {
        #[serde_as(as = "DisplayFromStr")]
        code: StatusCode,
        #[source]
        source: EntityError,
    },

    #[error("request timeout {:?}", .0)]
    #[serde(with = "self::serde_duration")]
    TimeoutError(Duration),

    #[error("not found api endpoint")]
    UnmatchedPathError,

    #[error("invalid username or password")]
    LoginFailError,
    #[error("inactive user")]
    InactiveUserError,
    #[error("login required")]
    LoginRequiredError,
}
impl ApiError {
    pub fn status_code(&self) -> &StatusCode {
        match self {
            Self::AnyhowError { code, .. } => code,
            Self::CannotConnectDatabase => &StatusCode::INTERNAL_SERVER_ERROR,
            Self::RecordNotFound => &StatusCode::NOT_FOUND,
            Self::UnexpectedDatabaseError { code, .. } => code,
            Self::EntityError { code, .. } => code,
            Self::TimeoutError(_) => &StatusCode::REQUEST_TIMEOUT,
            Self::UnmatchedPathError => &StatusCode::NOT_FOUND,
            Self::LoginFailError => &StatusCode::FORBIDDEN,
            Self::InactiveUserError => &StatusCode::FORBIDDEN,
            Self::LoginRequiredError => &StatusCode::FORBIDDEN,
        }
    }
    pub async fn handle_timeout(error: BoxError) -> impl IntoResponse {
        if error.is::<tower::timeout::error::Elapsed>() {
            // TODO state
            ApiError::TimeoutError(Config::timeout(&Default::default()).clone())
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
        let (code, response) = (self.status_code().clone(), ApiResponse::<()>::failure(self));
        (code, Json(json!(response))).into_response()
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        (StatusCode::INTERNAL_SERVER_ERROR, error).into()
    }
}
impl From<(StatusCode, anyhow::Error)> for ApiError {
    fn from((status, error): (StatusCode, anyhow::Error)) -> Self {
        Self::AnyhowError { code: status, source: error }
    }
}
impl From<sea_orm::DbErr> for ApiError {
    fn from(error: sea_orm::DbErr) -> Self {
        (StatusCode::INTERNAL_SERVER_ERROR, error).into()
    }
}
impl From<(StatusCode, sea_orm::DbErr)> for ApiError {
    fn from((status, error): (StatusCode, sea_orm::DbErr)) -> Self {
        match error {
            sea_orm::DbErr::Conn(_) => Self::CannotConnectDatabase,
            sea_orm::DbErr::RecordNotFound(_) => Self::RecordNotFound,
            _ => Self::UnexpectedDatabaseError { code: status },
        }
    }
}
impl From<EntityError> for ApiError {
    fn from(error: EntityError) -> Self {
        (StatusCode::BAD_REQUEST, error).into()
    }
}
impl From<(StatusCode, EntityError)> for ApiError {
    fn from((status, error): (StatusCode, EntityError)) -> Self {
        Self::EntityError { code: status, source: error }
    }
}

mod serde_anyhow {
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(err: &anyhow::Error, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        err.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<anyhow::Error, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let msg = <String>::deserialize(deserializer)?;
        Ok(anyhow::Error::msg(msg))
    }
}

// TODO use #[serde_as(as = "serde_with::DurationSeconds<i64>")], but do not #[derive(Clone)]
mod serde_duration {
    use chrono::Duration;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        (duration.to_std().unwrap()).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let duration = <std::time::Duration>::deserialize(deserializer)?;
        Ok(Duration::from_std(duration).unwrap())
    }
}
