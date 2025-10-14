use crate::shorty::model::shorty_model::GetUserIdByUrlIdModel;
use crate::shorty::repository::shorty_repository::ShortyRepository;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};

#[derive(Debug, thiserror::Error)]
pub enum DeleteUrlServiceError {
    #[error("Database error")]
    DbError,
}

pub struct DeleteUrlService {
    shorty_repository: ShortyRepository,
}

impl DeleteUrlService {
    pub fn new(shorty_repository: ShortyRepository) -> Self {
        Self { shorty_repository }
    }

    pub fn delete_url(&self, id: i64) -> Result<(), Report<DeleteUrlServiceError>> {
        self.shorty_repository
            .delete_url_redirect(id)
            .change_context(DeleteUrlServiceError::DbError)?;

        Ok(())
    }

    pub fn fetch_user_id_from_url_id(
        &self,
        id: i64,
    ) -> Result<GetUserIdByUrlIdModel, Report<DeleteUrlServiceError>> {
        self.shorty_repository
            .get_user_id_by_url_id(id)
            .change_context(DeleteUrlServiceError::DbError)?
            .ok_or_else(|| {
                Report::new(DeleteUrlServiceError::DbError).attach(StatusCode::NOT_FOUND)
            })
    }
}

impl FromContext for DeleteUrlService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shorty::repository::shorty_repository::ShortyRepositoryError;

    #[test]
    fn test_delete_url_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_delete_url_redirect(1)
            .returns_once(Ok(()));

        let delete_url_service = DeleteUrlService::new(shorty_repository);
        let result = delete_url_service.delete_url(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_delete_url_db_error() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_delete_url_redirect(1)
            .returns_once(Err(Report::new(ShortyRepositoryError::QueryError)));

        let delete_url_service = DeleteUrlService::new(shorty_repository);
        let result = delete_url_service.delete_url(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_fetch_user_id_from_url_id_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_user_id_by_url_id(1)
            .returns_once(Ok(Some(GetUserIdByUrlIdModel {
                created_by_user_id: 1,
            })));

        let delete_url_service = DeleteUrlService::new(shorty_repository);
        let user_id = delete_url_service.fetch_user_id_from_url_id(1).unwrap();
        assert_eq!(user_id.created_by_user_id, 1);
    }

    #[test]
    fn test_fetch_user_id_from_url_id_not_found() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_user_id_by_url_id(1)
            .returns_once(Ok(None));

        let delete_url_service = DeleteUrlService::new(shorty_repository);
        let user_id = delete_url_service.fetch_user_id_from_url_id(1);
        assert!(user_id.is_err());
        let error = user_id.as_ref().err().unwrap();
        let http_code = error.downcast_ref::<StatusCode>().unwrap();
        assert_eq!(http_code, &StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_fetch_user_id_from_url_id_db_error() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_user_id_by_url_id(1)
            .returns_once(Err(Report::new(ShortyRepositoryError::QueryError)));

        let delete_url_service = DeleteUrlService::new(shorty_repository);
        let user_id = delete_url_service.fetch_user_id_from_url_id(1);
        assert!(user_id.is_err());
    }
}
