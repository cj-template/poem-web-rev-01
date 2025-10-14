use crate::shorty::form::add_edit_url_form::AddEditUrlValidated;
use crate::shorty::model::shorty_model::{GetUrlRedirectModel, GetUserIdByUrlIdModel};
use crate::shorty::repository::shorty_repository::ShortyRepository;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};

#[derive(Debug, thiserror::Error)]
pub enum EditUrlServiceError {
    #[error("Database error")]
    DbError,
}

pub struct EditUrlService {
    shorty_repository: ShortyRepository,
}

impl EditUrlService {
    pub fn new(shorty_repository: ShortyRepository) -> Self {
        Self { shorty_repository }
    }

    pub fn get_url_redirect(
        &self,
        id: i64,
    ) -> Result<GetUrlRedirectModel, Report<EditUrlServiceError>> {
        self.shorty_repository
            .get_url_redirect(id)
            .change_context(EditUrlServiceError::DbError)?
            .ok_or_else(|| Report::new(EditUrlServiceError::DbError).attach(StatusCode::NOT_FOUND))
    }

    pub fn edit_url_submit(
        &self,
        form: &AddEditUrlValidated,
        id: i64,
    ) -> Result<(), Report<EditUrlServiceError>> {
        self.shorty_repository
            .edit_url_redirect(id, form.url_path.as_str(), form.url_redirect.as_str())
            .change_context(EditUrlServiceError::DbError)?;

        Ok(())
    }

    pub fn fetch_user_id_from_url_id(
        &self,
        id: i64,
    ) -> Result<GetUserIdByUrlIdModel, Report<EditUrlServiceError>> {
        self.shorty_repository
            .get_user_id_by_url_id(id)
            .change_context(EditUrlServiceError::DbError)?
            .ok_or_else(|| Report::new(EditUrlServiceError::DbError).attach(StatusCode::NOT_FOUND))
    }
}

impl FromContext for EditUrlService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shorty::form::add_edit_url_form::AddEditUrlForm;
    use crate::shorty::repository::shorty_repository::ShortyRepositoryError;

    #[test]
    fn test_get_url_redirect_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_url_redirect(1)
            .returns_once(Ok(Some(GetUrlRedirectModel {
                url_path: "hello".to_string(),
                url_redirect: "hi".to_string(),
            })));

        let edit_url_service = EditUrlService::new(shorty_repository);
        let url_redirect = edit_url_service.get_url_redirect(1).unwrap();
        assert_eq!(url_redirect.url_path, "hello");
        assert_eq!(url_redirect.url_redirect, "hi");
    }

    #[test]
    fn test_get_url_redirect_not_found() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_url_redirect(1)
            .returns_once(Ok(None));

        let edit_url_service = EditUrlService::new(shorty_repository);
        let url_redirect = edit_url_service.get_url_redirect(1);
        assert!(url_redirect.is_err());
        let error = url_redirect.as_ref().err().unwrap();
        let http_code = error.downcast_ref::<StatusCode>().unwrap();
        assert_eq!(http_code, &StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_edit_url_submit_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_edit_url_redirect(1, "hello", "http://hello.com")
            .returns_once(Ok(()));

        let edit_url_service = EditUrlService::new(shorty_repository);

        let mut add_edit_url_form = AddEditUrlForm::default();
        add_edit_url_form.url_path = "hello".to_string();
        add_edit_url_form.url_redirect = "http://hello.com".to_string();

        let validate = add_edit_url_form.as_validated().await.0.unwrap();

        let result = edit_url_service.edit_url_submit(&validate, 1);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_edit_url_submit_db_error() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_edit_url_redirect(1, "hello", "http://hello.com")
            .returns_once(Err(Report::new(ShortyRepositoryError::QueryError)));

        let edit_url_service = EditUrlService::new(shorty_repository);

        let mut add_edit_url_form = AddEditUrlForm::default();
        add_edit_url_form.url_path = "hello".to_string();
        add_edit_url_form.url_redirect = "http://hello.com".to_string();

        let validate = add_edit_url_form.as_validated().await.0.unwrap();

        let result = edit_url_service.edit_url_submit(&validate, 1);
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

        let edit_url_service = EditUrlService::new(shorty_repository);
        let user_id = edit_url_service.fetch_user_id_from_url_id(1).unwrap();
        assert_eq!(user_id.created_by_user_id, 1);
    }

    #[test]
    fn test_fetch_user_id_from_url_id_not_found() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_get_user_id_by_url_id(1)
            .returns_once(Ok(None));

        let edit_url_service = EditUrlService::new(shorty_repository);
        let user_id = edit_url_service.fetch_user_id_from_url_id(1);
        assert!(user_id.is_err());
        let error = user_id.as_ref().err().unwrap();
        let http_code = error.downcast_ref::<StatusCode>().unwrap();
        assert_eq!(http_code, &StatusCode::NOT_FOUND);
    }
}
