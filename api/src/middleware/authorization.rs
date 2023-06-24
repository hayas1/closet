use std::str::FromStr;

use axum::{extract::State, middleware::Next, response::Response};
use chrono::Utc;
use entity::{
    class::{email::Email, id::Id, username::Username},
    user,
};
use hyper::{header, http::HeaderValue, HeaderMap, Request};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, ActiveValue, DatabaseConnection, EntityTrait,
    IntoActiveModel,
};
use serde::{Deserialize, Serialize};

use crate::{response::error::ApiError, AppState, Configuration};

pub async fn verification<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> Response {
    let user = AuthUser::verificate(req.headers(), &state.db, &state.decoding_key).await;
    req.extensions_mut().insert(user);
    next.run(req).await
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
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
        let claims = TokenClaims { sub, iat, exp };
        let token = Some(
            jsonwebtoken::encode(&Header::default(), &claims, key)
                .map_err(|e| anyhow::anyhow!(e))?,
        );

        let mut active = user.into_active_model();
        active.last_login = ActiveValue::Set(Some(now.fixed_offset()));
        let updated = active.update(db);

        Ok(Self { token, ..updated.await?.into() })
    }
    pub async fn verificate(
        headers: &HeaderMap<HeaderValue>,
        db: &DatabaseConnection,
        key: &DecodingKey,
    ) -> Option<AuthUser> {
        const BEARER: &str = "Bearer ";
        let header_value = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
        let (bearer, token) = header_value.split_at(BEARER.len());
        (bearer == BEARER).then(|| ())?;

        let TokenClaims { sub, .. } =
            jsonwebtoken::decode::<TokenClaims>(&token, &key, &Validation::default()).ok()?.claims;

        let user =
            user::Entity::find_by_id(Id::<user::Model>::from_str(&sub).ok()?).one(db).await.ok()?;
        user.and_then(|u| Some(u.into()))
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
