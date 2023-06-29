use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
};

use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};

pub type Configuration = Arc<Config>;
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Config {
    pub host: Option<String>,
    pub port: Option<String>,
    pub base_url: Option<String>,
    pub timeout: Option<String>,
    pub secret_key: Option<String>,
    pub jwt_expired: Option<String>,
    pub database_url: Option<String>,
    pub mysql_host: Option<String>,
    pub mysql_user: Option<String>,
    pub mysql_password: Option<String>,
    pub mysql_port: Option<String>,
    pub mysql_db: Option<String>,
    pub migrate: Option<bool>,
}
// TODO refactor
impl Config {
    pub const HOST: &str = "HOST";
    pub const PORT: &str = "PORT";
    pub const BASE_URL: &str = "BASE_URL";
    pub const TIMEOUT: &str = "TIMEOUT";
    pub const SECRET_KEY: &str = "SECRET_KEY";
    pub const JWT_EXPIRED: &str = "EXPIRED";
    pub const DATABASE_URL: &str = "DATABASE_URL";
    pub const MYSQL_HOST: &str = "MYSQL_HOST";
    pub const MYSQL_USER: &str = "MYSQL_USER";
    pub const MYSQL_PASSWORD: &str = "MYSQL_PASSWORD";
    pub const MYSQL_PORT: &str = "MYSQL_PORT";
    pub const MYSQL_DB: &str = "MYSQL_DB";
    pub const MIGRATE: &str = "MIGRATE";

    pub fn environ() -> Self {
        Self::default()
    }
    pub fn last_resort() -> Self {
        Self {
            host: Some("0.0.0.0".into()),
            port: Some("3000".into()),
            base_url: Some("/".into()),
            timeout: Some("1000ms".into()),
            secret_key: Some("".into()),
            jwt_expired: Some("7d".into()),
            database_url: Some("mysql://root:root@localhost:3306/db".into()),
            mysql_user: Some("root".into()),
            mysql_password: Some("root".into()),
            mysql_host: Some("localhost".into()),
            mysql_port: Some("3306".into()),
            mysql_db: Some("db".into()),
            migrate: Some(false),
        }
    }

    pub fn address(&self) -> &SocketAddr {
        static _ADDRESS: OnceLock<SocketAddr> = OnceLock::new();
        _ADDRESS.get_or_init(|| {
            let Self { host, port, .. } = Self::last_resort();
            let (h, p) = (
                std::env::var(Self::HOST).unwrap_or(host.expect("last_resort")),
                std::env::var(Self::PORT).unwrap_or(port.expect("last_resort")),
            );
            SocketAddr::new(
                self.host.clone().unwrap_or(h).parse().unwrap_or_else(|e| panic!("{}", e)),
                self.port.clone().unwrap_or(p).parse().unwrap_or_else(|e| panic!("{}", e)),
            )
        })
    }

    pub fn base_url(&self) -> &str {
        static _BASE_URL: OnceLock<String> = OnceLock::new();
        _BASE_URL.get_or_init(|| {
            let Self { base_url, .. } = Self::last_resort();
            self.base_url
                .clone()
                .unwrap_or(std::env::var(Self::BASE_URL).unwrap_or(base_url.expect("last_resort")))
        })
    }

    pub fn timeout(&self) -> &Duration {
        static _TIMEOUT: OnceLock<Duration> = OnceLock::new();
        _TIMEOUT.get_or_init(|| {
            let Self { timeout, .. } = Self::last_resort();
            let ts = std::env::var(Self::TIMEOUT).unwrap_or(timeout.expect("last_resort"));
            let std_duration = duration_str::parse(&self.timeout.clone().unwrap_or(ts))
                .unwrap_or_else(|e| panic!("{:?}", e));
            chrono::Duration::from_std(std_duration).unwrap_or_else(|e| panic!("{}", e))
        })
    }

    pub fn secret_key(&self) -> &str {
        static _SECRET_KEY: OnceLock<String> = OnceLock::new();
        _SECRET_KEY.get_or_init(|| {
            let Self { secret_key, .. } = Self::last_resort();
            let secret = self.secret_key.clone().unwrap_or(
                std::env::var(Self::SECRET_KEY).unwrap_or(secret_key.expect("last_resort")),
            );
            if secret == Self::last_resort().secret_key.expect("last_resort") {
                panic!("must set: {}", Self::SECRET_KEY);
            } else {
                secret
            }
        })
    }
    pub fn encoding_key(&self) -> EncodingKey {
        EncodingKey::from_secret(self.secret_key().as_ref())
    }
    pub fn decoding_key(&self) -> DecodingKey {
        DecodingKey::from_secret(self.secret_key().as_ref())
    }

    pub fn jwt_expired(&self) -> &Duration {
        static _JWT_EXPIRED: OnceLock<Duration> = OnceLock::new();
        _JWT_EXPIRED.get_or_init(|| {
            let Self { jwt_expired, .. } = Self::last_resort();
            let exp = std::env::var(Self::JWT_EXPIRED).unwrap_or(jwt_expired.expect("last_resort"));
            let std_duration = duration_str::parse(&self.jwt_expired.clone().unwrap_or(exp))
                .unwrap_or_else(|e| panic!("{:?}", e));
            chrono::Duration::from_std(std_duration).unwrap_or_else(|e| panic!("{}", e))
        })
    }

    pub fn database_url(&self) -> &str {
        static _DATABASE_URL: OnceLock<String> = OnceLock::new();
        _DATABASE_URL.get_or_init(|| {
            let Self {
                database_url,
                mysql_user,
                mysql_password,
                mysql_host,
                mysql_port,
                mysql_db,
                ..
            } = Self::last_resort();
            let db = match std::env::var(Self::DATABASE_URL) {
                Ok(url) => url,
                Err(_) => format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.mysql_user.clone().unwrap_or(
                        std::env::var(Self::MYSQL_USER).unwrap_or(mysql_user.expect("last_resort"))
                    ),
                    self.mysql_password.clone().unwrap_or(
                        std::env::var(Self::MYSQL_PASSWORD)
                            .unwrap_or(mysql_password.expect("last_resort"))
                    ),
                    self.mysql_host.clone().unwrap_or(
                        std::env::var(Self::MYSQL_HOST).unwrap_or(mysql_host.expect("last_resort"))
                    ),
                    self.mysql_port.clone().unwrap_or(
                        std::env::var(Self::MYSQL_PORT).unwrap_or(mysql_port.expect("last_resort"))
                    ),
                    self.mysql_db.clone().unwrap_or(
                        std::env::var(Self::MYSQL_DB).unwrap_or(mysql_db.expect("last_resort"))
                    ),
                ),
            };
            let database = self.database_url.clone().unwrap_or(db);
            if database == database_url.expect("last_resort") {
                tracing::warn!("use last_resort {}: {}", Self::DATABASE_URL, database);
            }
            database
        })
    }

    pub fn migrate(&self) -> bool {
        static _MIGRATE: OnceLock<bool> = OnceLock::new();
        _MIGRATE
            .get_or_init(|| {
                let Self { migrate, .. } = Self::last_resort();
                self.migrate.clone().unwrap_or(
                    std::env::var(Self::MIGRATE)
                        .map(|s| s.to_lowercase() == "true") // TODO better way
                        .unwrap_or(migrate.expect("last_resort")),
                )
            })
            .clone()
    }
    // pub async fn database_connection(&self) -> DatabaseConnection {
    //     use tokio::sync::OnceCell;
    //     static _DATABASE_CONNECTION: OnceCell<DatabaseConnection> = OnceCell::const_new();
    //     _DATABASE_CONNECTION
    //         .get_or_init(|| async { Database::connect(self.database_url()).await.unwrap() }) // TODO error handling
    //         .await
    //         .clone()
    // }
}
