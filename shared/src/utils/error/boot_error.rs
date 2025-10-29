use thiserror::Error;

#[derive(Debug, Error)]
pub enum MainError {
    #[error("Config error")]
    ConfigError,
    #[error("IO error")]
    IoError,
    #[error("Locale error")]
    LocaleError,
    #[error("Thread error")]
    ThreadError,
}
