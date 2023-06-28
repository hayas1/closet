use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
};

use chrono::Duration;

pub type Configuration = Arc<Config>;
pub struct Config {
    host: Option<String>,
    port: Option<String>,
    base_url: Option<String>,
    timeout: Option<String>,
    secret_key: Option<String>,
    jwt_expired: Option<String>,
    database_url: Option<String>,
    mysql_host: Option<String>,
    mysql_user: Option<String>,
    mysql_password: Option<String>,
    mysql_port: Option<String>,
    mysql_db: Option<String>,
}
impl Default for Config {
    fn default() -> Self {
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
        }
    }
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

    pub fn address(&self) -> &SocketAddr {
        static _ADDRESS: OnceLock<SocketAddr> = OnceLock::new();
        _ADDRESS.get_or_init(|| {
            let Self { host, port, .. } = Self::default();
            let (h, p) = (
                std::env::var(Self::HOST).unwrap_or(host.expect("default")),
                std::env::var(Self::PORT).unwrap_or(port.expect("default")),
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
            let Self { base_url, .. } = Self::default();
            self.base_url
                .clone()
                .unwrap_or(std::env::var(Self::BASE_URL).unwrap_or(base_url.expect("default")))
        })
    }

    pub fn timeout(&self) -> &Duration {
        static _TIMEOUT: OnceLock<Duration> = OnceLock::new();
        _TIMEOUT.get_or_init(|| {
            let Self { timeout, .. } = Self::default();
            let ts = std::env::var(Self::TIMEOUT).unwrap_or(timeout.expect("default"));
            let std_duration = duration_str::parse(&self.timeout.clone().unwrap_or(ts))
                .unwrap_or_else(|e| panic!("{:?}", e));
            chrono::Duration::from_std(std_duration).unwrap_or_else(|e| panic!("{}", e))
        })
    }

    pub fn secret_key(&self) -> &str {
        static _SECRET_KEY: OnceLock<String> = OnceLock::new();
        _SECRET_KEY.get_or_init(|| {
            let Self { secret_key, .. } = Self::default();
            let secret = self
                .secret_key
                .clone()
                .unwrap_or(std::env::var(Self::SECRET_KEY).unwrap_or(secret_key.expect("default")));
            if secret == Self::default().secret_key.expect("default") {
                panic!("must be set: {}", Self::SECRET_KEY);
            } else {
                secret
            }
        })
    }

    pub fn jwt_expired(&self) -> &Duration {
        static _JWT_EXPIRED: OnceLock<Duration> = OnceLock::new();
        _JWT_EXPIRED.get_or_init(|| {
            let Self { jwt_expired, .. } = Self::default();
            let exp = std::env::var(Self::JWT_EXPIRED).unwrap_or(jwt_expired.expect("default"));
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
            } = Self::default();
            let db = match std::env::var(Self::DATABASE_URL) {
                Ok(url) => url,
                Err(_) => format!(
                    "mysql://{}:{}@{}:{}/{}",
                    self.mysql_user.clone().unwrap_or(
                        std::env::var(Self::MYSQL_USER).unwrap_or(mysql_user.expect("default"))
                    ),
                    self.mysql_password.clone().unwrap_or(
                        std::env::var(Self::MYSQL_PASSWORD)
                            .unwrap_or(mysql_password.expect("default"))
                    ),
                    self.mysql_host.clone().unwrap_or(
                        std::env::var(Self::MYSQL_HOST).unwrap_or(mysql_host.expect("default"))
                    ),
                    self.mysql_port.clone().unwrap_or(
                        std::env::var(Self::MYSQL_PORT).unwrap_or(mysql_port.expect("default"))
                    ),
                    self.mysql_db.clone().unwrap_or(
                        std::env::var(Self::MYSQL_DB).unwrap_or(mysql_db.expect("default"))
                    ),
                ),
            };
            let database = self.database_url.clone().unwrap_or(db);
            if database == database_url.expect("default") {
                tracing::warn!("use default {}: {}", Self::DATABASE_URL, database);
            }
            database
        })
    }
}
