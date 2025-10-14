use crate::shorty::model::url::UrlRedirect;
use crate::shorty::repository::shorty::ShortyRepository;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};
use shared::error::ExtraResultExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FetchUrlServiceError {
    #[error("Db error")]
    DbError,
    #[error("Not Found")]
    NotFound,
}

pub struct FetchUrlService {
    shorty_repository: ShortyRepository,
}

impl FetchUrlService {
    pub fn new(shorty_repository: ShortyRepository) -> Self {
        Self { shorty_repository }
    }

    pub fn fetch_url(&self, path: &str) -> Result<UrlRedirect, Report<FetchUrlServiceError>> {
        let url_redirect = self
            .shorty_repository
            .fetch_url(path)
            .change_context(FetchUrlServiceError::DbError)
            .log_it()?
            .ok_or_else(|| {
                Report::new(FetchUrlServiceError::NotFound)
                    .attach(format!("Path: {}", path))
                    .attach(StatusCode::NOT_FOUND)
            })?;
        Ok(url_redirect)
    }
}

impl FromContext for FetchUrlService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shorty::repository::shorty::ShortyRepositoryError;

    #[test]
    fn test_fetch_url_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_fetch_url("hello")
            .returns_once(Ok(Some(UrlRedirect {
                url_redirect: "hi".to_string(),
            })));

        let fetch_url_service = FetchUrlService::new(shorty_repository);
        let url_redirect = fetch_url_service.fetch_url("hello").unwrap();
        assert_eq!(url_redirect.url_redirect, "hi");
    }

    #[test]
    fn test_fetch_url_not_found() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_fetch_url("hello")
            .returns_once(Ok(None));

        let fetch_url_service = FetchUrlService::new(shorty_repository);
        let url_redirect = fetch_url_service.fetch_url("hello");
        assert!(url_redirect.is_err());
        let error = url_redirect.as_ref().err().unwrap();
        let http_code = error.downcast_ref::<StatusCode>().unwrap();
        assert_eq!(http_code, &StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_fetch_url_db_error() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_fetch_url("hello")
            .returns_once(Err(Report::new(ShortyRepositoryError::RowValueError)));
        let fetch_url_service = FetchUrlService::new(shorty_repository);
        let url_redirect = fetch_url_service.fetch_url("hello");
        assert!(url_redirect.is_err());
    }
}
