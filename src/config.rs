//! 環境変数から読む設定。読み取りと検証をここに集約する。

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::FromStr;

/// `RUST_LOG` が未設定のときのログレベル。
pub const DEFAULT_LOG_FILTER: &str = "todo_app=debug,tower_http=info,sqlx=warn";

const DEFAULT_HOST: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);
const DEFAULT_PORT: u16 = 3000;
const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_LOG_DIR: &str = "logs";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0} is not set")]
    Missing(&'static str),

    #[error("{key} is invalid: {value:?} ({reason})")]
    Invalid {
        key: &'static str,
        value: String,
        reason: String,
    },
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub max_connections: u32,
    pub bind_addr: SocketAddr,
    pub log_dir: String,
    pub log_filter: String,
}

impl Config {
    /// プロセスの環境変数から読む。
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_source(|key| std::env::var(key).ok())
    }

    /// 環境変数の取得方法を差し替えられる形。テストはこちらを使う。
    fn from_source(get: impl Fn(&'static str) -> Option<String>) -> Result<Self, ConfigError> {
        let database_url = get("DATABASE_URL").ok_or(ConfigError::Missing("DATABASE_URL"))?;

        let host = parse_or("HOST", get("HOST"), DEFAULT_HOST)?;
        let port = parse_or("PORT", get("PORT"), DEFAULT_PORT)?;
        let max_connections = parse_or(
            "DATABASE_MAX_CONNECTIONS",
            get("DATABASE_MAX_CONNECTIONS"),
            DEFAULT_MAX_CONNECTIONS,
        )?;

        Ok(Config {
            database_url,
            max_connections,
            bind_addr: SocketAddr::new(host, port),
            log_dir: get("LOG_DIR").unwrap_or_else(|| DEFAULT_LOG_DIR.to_string()),
            log_filter: get("RUST_LOG").unwrap_or_else(|| DEFAULT_LOG_FILTER.to_string()),
        })
    }
}

/// 未設定なら既定値、設定されていれば解釈する。解釈できなければどのキーが悪いか示す。
fn parse_or<T>(key: &'static str, value: Option<String>, default: T) -> Result<T, ConfigError>
where
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let Some(raw) = value else {
        return Ok(default);
    };

    match raw.parse() {
        Ok(parsed) => Ok(parsed),
        Err(err) => Err(ConfigError::Invalid {
            key,
            value: raw,
            reason: err.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 指定したキーだけを返す取得関数を作る。
    fn source(pairs: &[(&str, &str)]) -> impl Fn(&'static str) -> Option<String> + use<> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        move |key| pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    }

    #[test]
    fn database_url_is_required() {
        let err = Config::from_source(source(&[])).unwrap_err();

        assert_eq!(err.to_string(), "DATABASE_URL is not set");
    }

    #[test]
    fn defaults_are_applied_when_only_database_url_is_set() {
        let config = Config::from_source(source(&[("DATABASE_URL", "postgres://localhost/x")]))
            .expect("should build with defaults");

        assert_eq!(config.bind_addr.to_string(), "127.0.0.1:3000");
        assert_eq!(config.max_connections, 5);
        assert_eq!(config.log_dir, "logs");
        assert_eq!(config.log_filter, DEFAULT_LOG_FILTER);
    }

    #[test]
    fn values_from_the_environment_win() {
        let config = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/x"),
            ("HOST", "0.0.0.0"),
            ("PORT", "8080"),
            ("DATABASE_MAX_CONNECTIONS", "20"),
            ("LOG_DIR", "/var/log/todo"),
            ("RUST_LOG", "info"),
        ]))
        .expect("should accept every value");

        assert_eq!(config.bind_addr.to_string(), "0.0.0.0:8080");
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.log_dir, "/var/log/todo");
        assert_eq!(config.log_filter, "info");
    }

    #[test]
    fn an_unparsable_port_names_the_offending_key() {
        let err = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/x"),
            ("PORT", "http"),
        ]))
        .unwrap_err();

        assert!(
            err.to_string().starts_with("PORT is invalid: \"http\""),
            "{err}"
        );
    }

    #[test]
    fn an_unparsable_host_names_the_offending_key() {
        let err = Config::from_source(source(&[
            ("DATABASE_URL", "postgres://localhost/x"),
            ("HOST", "localhost"),
        ]))
        .unwrap_err();

        assert!(err.to_string().starts_with("HOST is invalid"), "{err}");
    }
}
