use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum EntityError {
    #[error("{0}")]
    ValidateError(#[source] ValidateError),

    #[error("fail to hash password")]
    CannotHashPassword,
}
impl From<ValidateError> for EntityError {
    fn from(inner: ValidateError) -> Self {
        Self::ValidateError(inner)
    }
}
impl From<EntityError> for sea_orm::TryGetError {
    fn from(e: EntityError) -> Self {
        match e {
            EntityError::ValidateError(e) => Self::DbErr(sea_orm::DbErr::Type(e.to_string())),
            EntityError::CannotHashPassword => Self::DbErr(sea_orm::DbErr::Custom(e.to_string())),
        }
    }
}
impl From<EntityError> for sea_orm::sea_query::ValueTypeErr {
    fn from(_: EntityError) -> Self {
        Self
    }
}

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
    #[error("unrecognized status")]
    UnrecognizedStatus,
    #[error("cannot convert to string")]
    CannotConvertToString,
}
