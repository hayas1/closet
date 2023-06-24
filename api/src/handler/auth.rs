use axum::{extract::Json, extract::State, Extension, Router};
use chrono::Utc;
use entity::{
    class::{password::RawPassword, username::Username},
    user::{self, NewUser},
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;

use crate::{
    middleware::authorization::AuthUser,
    response::{error::ApiError, message::Either, result::ApiResponse, ApiResult},
    AppState,
};

pub fn auth_router() -> Router<AppState> {
    axum::Router::new()
        .route("/create", axum::routing::post(create))
        .route("/login", axum::routing::post(login))
        .route("/whoami", axum::routing::get(whoami))
        .route("/logout", axum::routing::post(logout))
        .route("/delete", axum::routing::post(delete))
}

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
            <String as TryInto<RawPassword>>::try_into(password)?.hashed()?,
            display_name,
        );
        Ok(NewUser { email, username, password, display_name })
    }
}
pub async fn create(
    State(state): State<AppState>,
    Json(schema): Json<UserCreate>,
) -> ApiResult<AuthUser> {
    let new_user: NewUser = schema.try_into()?;
    let created = new_user.into_active_model().insert(&state.db);
    Ok(ApiResponse::new(created.await?.into()))
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub username: String,
    pub password: String,
}
pub async fn login(
    State(state): State<AppState>,
    Json(schema): Json<UserLogin>,
) -> ApiResult<AuthUser> {
    let (username, password) = (
        <String as TryInto<Username>>::try_into(schema.username)?,
        <String as TryInto<RawPassword>>::try_into(schema.password)?,
    );
    let login_user = entity::user::Entity::find()
        .filter(entity::user::Column::Username.eq(username))
        .one(&state.db)
        .await
        .unwrap_or(None)
        .ok_or_else(|| ApiError::LoginFailError)?;
    if !login_user.password.verify(password) {
        Err(ApiError::LoginFailError)?
    }

    let user = AuthUser::authenticate(login_user, &state.db, &state.encoding_key);
    Ok(ApiResponse::new(user.await?))
}

pub async fn whoami(Extension(user): Extension<Option<AuthUser>>) -> ApiResult<Option<AuthUser>> {
    Ok(ApiResponse::new(user))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(user): Extension<Option<AuthUser>>,
) -> ApiResult<AuthUser> {
    // FIXME verificate and record, access to db twice
    let mut active = user::Entity::find_by_id(user.ok_or_else(|| ApiError::LoginRequiredError)?.id)
        .one(&state.db)
        .await?
        .expect("login user must exist")
        .into_active_model();
    active.last_logout = ActiveValue::Set(Some(Utc::now().fixed_offset()));
    let logout = active.update(&state.db);
    Ok(ApiResponse::new(logout.await?.into()))
}

pub async fn delete(State(state): State<AppState>) -> ApiResult<Either> {
    todo!()
}
