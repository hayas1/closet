use chrono::Utc;
use entity::{
    class::{email::Email, id::Id, username::Username},
    user,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ActiveValue, DatabaseConnection,
    IntoActiveModel,
};
use serde::{Deserialize, Serialize};

use crate::{response::error::ApiError, Configuration};

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaim {
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AuthUser {
    pub id: Id<user::Model>,
    pub username: Username,
    pub email: Email,
    pub token: Option<String>,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub last_login: Option<DateTimeWithTimeZone>,
}
impl AuthUser {
    pub async fn authenticate(
        user: user::Model,
        db: &DatabaseConnection,
        key: &EncodingKey,
    ) -> Result<Self, ApiError> {
        let now = Utc::now();
        let (sub, iat, exp) = (
            user.id.to_string(),
            now.timestamp(),
            (now + Configuration::jwt_expired().clone()).timestamp(),
        );
        let claim = TokenClaim { sub, iat, exp };
        let token = Some(encode(&Header::default(), &claim, key).map_err(|e| anyhow::anyhow!(e))?);

        let mut active = user.into_active_model();
        active.last_login = ActiveValue::Set(Some(now.fixed_offset()));
        let updated = active.update(db);

        Ok(Self { token, ..updated.await?.into() })
    }
}
impl From<user::Model> for AuthUser {
    fn from(
        user::Model {
            id,
            username,
            email,
            display_name,
            is_active,
            created_at,
            updated_at,
            last_login,
            ..
        }: user::Model,
    ) -> Self {
        let token = None;
        Self {
            id,
            username,
            email,
            token,
            display_name,
            is_active,
            created_at,
            updated_at,
            last_login,
        }
    }
}
