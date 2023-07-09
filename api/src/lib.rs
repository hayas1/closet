use configuration::Configuration;

pub mod configuration;
pub mod dev_debug;
pub mod handler;
pub mod middleware;
pub mod response;

pub fn api_router() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", axum::routing::get(handler::health::health))
        .nest("/dev/debug", dev_debug::dev_debug_router())
        .nest("/health", handler::health::health_router())
        .nest("/auth", handler::auth::auth_router())
        .route("/*404", axum::routing::any(response::error::ApiError::handle_not_found))
}
#[derive(Clone)]
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,
    pub configuration: configuration::Configuration,
}
pub async fn with_auth(
    router: axum::Router<AppState>,
    configuration: Configuration,
) -> Result<axum::Router, sea_orm::DbErr> {
    let db = sea_orm::Database::connect(configuration.database_url()).await?;
    if configuration.migrate() {
        use migration::{Migrator, MigratorTrait};
        Migrator::up(&db, None).await?;
    }
    let state = AppState { db, configuration };
    let timeout = state.clone().configuration.timeout().to_std().unwrap(); // TODO error handling

    Ok(axum::Router::new()
        .nest(&state.configuration.base_url(), router)
        .with_state(state.clone())
        .layer(axum::middleware::from_fn_with_state(state, middleware::authorization::verification))
        .layer(
            tower::ServiceBuilder::new()
                .layer(axum::error_handling::HandleErrorLayer::new(
                    response::error::ApiError::handle_timeout,
                ))
                .timeout(timeout),
        )
        .layer(axum::middleware::from_fn(middleware::logging::request_log)))
}

#[cfg(all(test, feature = "sqlite"))]
pub fn standalone() -> configuration::Config {
    use rand::distributions::{Alphanumeric, DistString};

    configuration::Config {
        database_url: Some("sqlite::memory:".into()),
        secret_key: Some(Alphanumeric.sample_string(&mut rand::thread_rng(), 1024)),
        migrate: Some(true),
        timeout: Some("1d".into()),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use entity::class::status::Status;
    use hyper::{body::to_bytes, Body, Request, StatusCode};
    use sea_orm::DatabaseConnection;
    use tower::ServiceExt;

    use crate::response::result::ApiResponse;

    use super::*;

    #[tokio::test]
    async fn test_health_call() {
        let (uri, body) = ("/health", Body::empty());
        let api = api_router().with_state(AppState {
            db: DatabaseConnection::Disconnected,
            configuration: Configuration::new(Default::default()),
        });
        let request = Request::builder().uri(uri).body(body).unwrap();
        let response = api.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":"ok"}"#);
        let health: ApiResponse<Status> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health.result(), &Status::Ok);
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_rich_health_call() {
        use crate::handler::health::RichHealth;

        let (uri, body) = ("/api/health/rich", Body::empty());
        let api = with_auth(
            api_router(),
            configuration::Configuration::new(configuration::Config {
                base_url: Some("/api".into()),
                ..standalone()
            }),
        );
        let request = Request::builder().uri(uri).body(body).unwrap();
        let response = api.await.unwrap().oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&bytes[..], br#"{"result":{"status":"ok"}}"#);
        let health: ApiResponse<RichHealth> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(health.result(), &RichHealth { status: Status::Ok });
    }

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_login_logout_scenario() {
        use entity::class::password::Password;

        use crate::{
            handler::auth::{UserCreate, UserLogin},
            middleware::authorization::AuthUser,
            response::error::ApiError,
        };

        let api =
            with_auth(api_router(), configuration::Configuration::new(standalone())).await.unwrap();

        let create = UserCreate {
            display_name: "hogehoge".into(),
            email: "hoge@fuga.piyo".into(),
            username: "fugafuga".into(),
            password: "piyopiyo".into(),
        };
        let created_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method(hyper::Method::POST)
                    .uri("/auth/create")
                    .header(hyper::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::json!(create).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let created_user: ApiResponse<AuthUser> =
            serde_json::from_slice(&to_bytes(created_response.into_body()).await.unwrap()).unwrap();
        assert!(matches!(created_user.result().token, None));
        assert_eq!(created_user.result().user.display_name, "hogehoge");
        assert_eq!(created_user.result().user.username.to_string(), "fugafuga");
        assert_eq!(created_user.result().user.email.to_string(), "hoge@fuga.piyo");
        assert_eq!(created_user.result().user.password, Password::Unauthenticated);

        let invalid = UserLogin { username: "fugafuga".into(), password: "pw".into() };
        let forbidden_login_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method(hyper::Method::POST)
                    .uri("/auth/login")
                    .header(hyper::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::json!(invalid).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let forbidden_login: serde_json::Value =
            serde_json::from_slice(&to_bytes(forbidden_login_response.into_body()).await.unwrap())
                .unwrap();
        assert!(matches!(
            // TODO deserialize error
            serde_json::from_value(forbidden_login["error"]["serde"].clone().into()).unwrap(),
            ApiError::LoginFailError
        ));

        let login = UserLogin { username: "fugafuga".into(), password: "piyopiyo".into() };
        let login_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .method(hyper::Method::POST)
                    .uri("/auth/login")
                    .header(hyper::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::from(serde_json::json!(login).to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let auth_user: ApiResponse<AuthUser> =
            serde_json::from_slice(&to_bytes(login_response.into_body()).await.unwrap()).unwrap();
        assert!(matches!(auth_user.result().token, Some(_)));
        assert_eq!(auth_user.result().user.display_name, "hogehoge");
        assert_eq!(auth_user.result().user.username.to_string(), "fugafuga");
        assert_eq!(auth_user.result().user.email.to_string(), "hoge@fuga.piyo");
        assert_eq!(auth_user.result().user.password, Password::Unauthenticated);
        assert_ne!(created_user.result().user, auth_user.result().user);
        assert_eq!(
            entity::model::user::Model {
                updated_at: Default::default(),
                ..created_user.result().user.clone()
            },
            entity::model::user::Model {
                last_login: None,
                updated_at: Default::default(),
                ..auth_user.result().user.clone()
            }
        );

        let access_token = auth_user.result().token.as_ref().unwrap();
        let whoami_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/whoami")
                    .header(hyper::header::AUTHORIZATION, format!("Bearer {}", access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let whoami_user: ApiResponse<AuthUser> =
            serde_json::from_slice(&to_bytes(whoami_response.into_body()).await.unwrap()).unwrap();
        assert!(matches!(whoami_user.result().token, Some(_)));
        assert_eq!(whoami_user.result().user.display_name, "hogehoge");
        assert_eq!(whoami_user.result().user.username.to_string(), "fugafuga");
        assert_eq!(whoami_user.result().user.email.to_string(), "hoge@fuga.piyo");
        assert_eq!(whoami_user.result().user.password, Password::Unauthenticated);
        assert_eq!(auth_user.result(), whoami_user.result());

        let logout_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/logout")
                    .method(hyper::Method::POST)
                    .header(hyper::header::AUTHORIZATION, format!("Bearer {}", access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let logout_user: ApiResponse<AuthUser> =
            serde_json::from_slice(&to_bytes(logout_response.into_body()).await.unwrap()).unwrap();
        assert!(matches!(logout_user.result().token, None));
        assert_eq!(logout_user.result().user.display_name, "hogehoge");
        assert_eq!(logout_user.result().user.username.to_string(), "fugafuga");
        assert_eq!(logout_user.result().user.email.to_string(), "hoge@fuga.piyo");
        assert_eq!(logout_user.result().user.password, Password::Unauthenticated);
        assert_ne!(whoami_user.result(), logout_user.result());
        assert_eq!(
            entity::model::user::Model {
                updated_at: Default::default(),
                ..whoami_user.result().user.clone()
            },
            entity::model::user::Model {
                last_logout: None,
                updated_at: Default::default(),
                ..logout_user.result().user.clone()
            }
        );

        let no_auth_response = api
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/auth/whoami")
                    .header(hyper::header::AUTHORIZATION, format!("Bearer {}", access_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let no_auth_whoami: ApiResponse<Option<AuthUser>> =
            serde_json::from_slice(&to_bytes(no_auth_response.into_body()).await.unwrap()).unwrap();
        assert!(matches!(no_auth_whoami.result(), None));
    }
}
