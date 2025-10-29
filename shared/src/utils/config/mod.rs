use crate::utils::context::{Context, ContextError, FromContext};
use error_stack::{FutureExt, Report, ResultExt};
use figment::providers::{Format, Serialized, Toml};
use figment::{Figment, Profile};
use poem::PoemConfig;
use serde::{Deserialize, Serialize};
use sqlite::SqliteConfig;
use std::env::var;
use std::ops::Deref;
use std::sync::{Arc, Weak};
use thiserror::Error;
use tokio::sync::OnceCell;

pub mod poem;
pub mod sqlite;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config did not parse")]
    ParseError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub poem_public: Arc<PoemConfig>,
    pub poem_backoffice: Arc<PoemConfig>,
    pub sqlite: Arc<SqliteConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            poem_public: Arc::new(PoemConfig::default()),
            poem_backoffice: Arc::new(PoemConfig {
                address: "127.0.0.1".to_string(),
                port: 8001,
            }),
            sqlite: Arc::new(SqliteConfig::default()),
        }
    }
}

static CONFIG_CACHE: OnceCell<Arc<Config>> = OnceCell::const_new();

impl Config {
    fn build_figment() -> Figment {
        let name_kebab_case = option_env!("ENV_PROJECT_NAME").unwrap_or("app");
        let name_case_case_upper = option_env!("ENV_PROJECT_NAME_SHOUT_SNAKE").unwrap_or("APP");

        Figment::new()
            .merge(Serialized::defaults(Config::default()))
            .merge(Toml::file(format!("{}.toml", name_kebab_case)).nested())
            .merge(
                Toml::file(
                    var(format!("{}_CONFIG_PATH", name_case_case_upper))
                        .unwrap_or_else(|_| format!("{}.local.toml", name_kebab_case)),
                )
                .nested(),
            )
            .select(Profile::from_env_or(
                format!("{}_PROFILE", name_case_case_upper).as_str(),
                "default",
            ))
    }

    fn parse() -> Result<Self, Report<ConfigError>> {
        Self::build_figment()
            .extract::<Self>()
            .change_context(ConfigError::ParseError)
    }

    pub async fn fetch() -> Result<Weak<Config>, Report<ConfigError>> {
        let config: Result<&Arc<Config>, Report<ConfigError>> = CONFIG_CACHE
            .get_or_try_init(|| async {
                let config = Self::parse()?;
                Ok(Arc::new(config))
            })
            .await;

        Ok(Arc::downgrade(config?))
    }
}

pub struct ConfigPointer(Arc<Config>);

impl Deref for ConfigPointer {
    type Target = Config;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for ConfigPointer {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl FromContext for ConfigPointer {
    async fn from_context(_ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let config = Config::fetch()
            .change_context(ContextError::ConfigError)
            .await?;
        let config = config
            .upgrade()
            .ok_or_else(|| Report::new(ContextError::ConfigError).attach("Config not found"))?;
        Ok(Self(config))
    }
}
