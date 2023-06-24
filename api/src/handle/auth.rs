use axum::{extract::Json, extract::State, Router};
use entity::{
    class::{email::Email, id::Id, password::RawPassword, username::Username},
    user::{self, NewUser},
};
use hyper::StatusCode;
use sea_orm::{
    prelude::DateTimeWithTimeZone, ActiveModelTrait, EntityTrait, FromJsonQueryResult,
    FromQueryResult, IntoActiveModel,
};
use serde::{Deserialize, Serialize};

use crate::{
    response::{message::Either, result::ApiResponse, ApiResult},
    AppState,
};

#[derive(Deserialize)]
pub struct UserCreate {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
}
impl TryFrom<UserCreate> for NewUser {
    type Error = entity::error::validate::ValidateError;
    fn try_from(
        UserCreate { email, username, password, display_name }: UserCreate,
    ) -> Result<Self, Self::Error> {
        let (email, username, password, display_name) = (
            email.try_into()?,
            username.try_into()?,
            <std::string::String as TryInto<RawPassword>>::try_into(password)?.hashed()?,
            display_name,
        );
        Ok(NewUser { email, username, password, display_name })
    }
}

#[derive(
    Debug, Clone, Eq, PartialEq, FromQueryResult, FromJsonQueryResult, Serialize, Deserialize,
)]
pub struct AuthUser {
    pub id: Id<user::Model>,
    pub username: Username,
    pub email: Email,
    pub display_name: String,
    pub is_active: bool,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    pub last_login: Option<DateTimeWithTimeZone>,
}
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Logout {
    pub msg: String,
}
pub fn auth_router() -> Router<AppState> {
    axum::Router::new().route("/create", axum::routing::post(create))
    // .route("/login", axum::routing::post(login))
    // .route("/whoami", axum::routing::get(whoami))
    // .route("/logout", axum::routing::delete(logout))
    // .route("/delete", axum::routing::delete(delete))
}
pub async fn create(
    State(state): State<AppState>,
    Json(create): Json<UserCreate>,
) -> ApiResult<AuthUser> {
    let new_user: NewUser =
        create.try_into().map_err(|e: <NewUser as TryFrom<UserCreate>>::Error| {
            (StatusCode::BAD_REQUEST, anyhow::anyhow!(e))
        })?;
    let created = new_user.into_active_model().insert(&state.db).await?;
    let user = entity::user::Entity::find_by_id(created.id).into_model().one(&state.db);
    Ok(ApiResponse::new(user.await?.expect("already created")))
}
