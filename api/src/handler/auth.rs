use axum::{extract::Json, extract::State, Extension, Router};
use chrono::Utc;
use entity::{
    class::{password::Password, username::Username},
    user::InsertUser,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
};
use serde::Deserialize;

use crate::{
    middleware::authorization::AuthUser,
    response::{error::ApiError, result::ApiResponse, ApiResult},
    AppState,
};

pub fn auth_router() -> Router<AppState> {
    axum::Router::new()
        .route("/create", axum::routing::post(create))
        .route("/login", axum::routing::post(login))
        .route("/whoami", axum::routing::get(whoami))
        // TODO .route("/confirm/:token", axum::routing::get(confirm))
        // TODO .route("/update", axum::routing::get(update))
        // TODO .route("/update/password", axum::routing::get(update_password))
        .route("/logout", axum::routing::post(logout))
        .route("/deactivate", axum::routing::post(deactivate))
}

#[derive(Deserialize)]
pub struct UserCreate {
    pub email: String,
    pub username: String,
    pub password: String,
    pub display_name: String,
}
impl TryFrom<UserCreate> for InsertUser {
    type Error = entity::error::validate::ValidateError;
    fn try_from(
        UserCreate { email, username, password, display_name }: UserCreate,
    ) -> Result<Self, Self::Error> {
        let (email, username, display_name) =
            (email.try_into()?, username.try_into()?, display_name);
        let password = Password::hash(password.as_bytes())?;
        let is_active = true;
        Ok(InsertUser { email, username, password, display_name, is_active })
    }
}
pub async fn create(
    State(state): State<AppState>,
    Json(schema): Json<UserCreate>,
) -> ApiResult<AuthUser> {
    let insert_user: InsertUser = schema.try_into()?;
    let created = insert_user.into_active_model().insert(&state.db);
    Ok(ApiResponse::new(AuthUser::new(None, created.await?)))
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
    let (username, raw) = (Username::parse(&schema.username)?, schema.password.as_bytes());
    let user = entity::user::Entity::find()
        .filter(entity::user::Column::Username.eq(username))
        .one(&state.db)
        .await
        .unwrap_or(None)
        .ok_or_else(|| ApiError::LoginFailError)?;
    if !user.password.verify(raw) {
        Err(ApiError::LoginFailError)?
    } else if !user.is_active {
        Err(ApiError::InactiveUserError)?
    }

    let login = AuthUser::authenticate(user, &state.db, &state.encoding_key);
    Ok(ApiResponse::new(login.await?))
}

pub async fn whoami(Extension(user): Extension<Option<AuthUser>>) -> ApiResult<Option<AuthUser>> {
    Ok(ApiResponse::new(user))
}

pub async fn logout(
    State(state): State<AppState>,
    Extension(user): Extension<Option<AuthUser>>,
) -> ApiResult<AuthUser> {
    // FIXME verificate and record, access to db twice
    let mut active = user.ok_or_else(|| ApiError::LoginRequiredError)?.into_active_model();
    active.last_logout = ActiveValue::Set(Some(Utc::now().fixed_offset()));
    let logout = active.update(&state.db);
    Ok(ApiResponse::new(AuthUser::new(None, logout.await?)))
}

pub async fn deactivate(
    State(state): State<AppState>,
    Extension(user): Extension<Option<AuthUser>>,
) -> ApiResult<AuthUser> {
    let mut active = user.ok_or_else(|| ApiError::LoginRequiredError)?.into_active_model();
    active.is_active = ActiveValue::Set(false);
    let deactivated = active.update(&state.db);
    Ok(ApiResponse::new(AuthUser::new(None, deactivated.await?)))
}
