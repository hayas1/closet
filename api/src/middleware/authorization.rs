use std::str::FromStr;

use axum::{extract::State, middleware::Next, response::Response};
use chrono::Utc;
use entity::{
    class::id::Id,
    user::{self, ActiveModel},
};
use hyper::{header, http::HeaderValue, HeaderMap, Request};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel,
    QueryFilter,
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
    pub token: Option<String>,
    pub user: user::Model,
}
impl AuthUser {
    pub fn new(token: Option<String>, user: user::Model) -> Self {
        let user = user.empty_password();
        Self { token, user }
    }
    pub fn into_active_model(self) -> ActiveModel {
        self.user.into_active_model()
    }

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
        let token = jsonwebtoken::encode(&Header::default(), &claims, key)
            .map_err(|e| anyhow::anyhow!(e))?;

        let mut active = user.into_active_model();
        active.last_login = ActiveValue::Set(Some(now.fixed_offset()));
        let user = active.update(db).await?;

        Ok(Self::new(Some(token), user))
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

        let found = user::Entity::find_by_id(Id::<user::Model>::from_str(&sub).ok()?)
            .filter(user::Column::IsActive.eq(true))
            .one(db)
            .await
            .ok()?;
        let user = found.filter(|u| {
            // if last_login <= last_logout <= now, do not verificate
            !((u.last_login.unwrap_or_default() <= u.last_logout.unwrap_or_default())
                && (u.last_logout.unwrap_or_default() <= Utc::now().fixed_offset()))
        })?;

        Some(Self::new(Some(token.into()), user))
    }
}
