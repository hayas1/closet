use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum ValidateError {
    #[error("token {} is invalid", invalid_token)]
    CannotValidateToken68 { invalid_token: String },
    #[error("email {} is invalid", invalid_email)]
    CannotValidateEmail { invalid_email: String },
    #[error("username {} is invalid", invalid_username)]
    CannotValidateUsername { invalid_username: String },
    #[error("password contains unavailable character")]
    ContainUnavailableCharacterPassword,
    #[error("fail to hash password")]
    CannotHashPassword,
}

impl From<ValidateError> for sea_orm::TryGetError {
    fn from(e: ValidateError) -> Self {
        Self::DbErr(sea_orm::DbErr::Type(e.to_string()))
    }
}

impl From<ValidateError> for sea_orm::sea_query::ValueTypeErr {
    fn from(_: ValidateError) -> Self {
        Self
    }
}
