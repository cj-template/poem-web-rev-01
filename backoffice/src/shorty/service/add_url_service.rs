use crate::shorty::form::add_edit_url_form::AddEditUrlValidated;
use crate::shorty::repository::shorty_repository::ShortyRepository;
use error_stack::{Report, ResultExt};
use shared::context::{Context, ContextError, FromContext};

#[derive(Debug, thiserror::Error)]
pub enum AddUrlServiceError {
    #[error("Database error")]
    DbError,
}

pub struct AddUrlService {
    shorty_repository: ShortyRepository,
}

impl AddUrlService {
    pub fn new(shorty_repository: ShortyRepository) -> Self {
        Self { shorty_repository }
    }

    pub fn add_url_submit(
        &self,
        form: &AddEditUrlValidated,
        user_id: i64,
    ) -> Result<(), Report<AddUrlServiceError>> {
        self.shorty_repository
            .add_url_redirect(form.url_path.as_str(), form.url_redirect.as_str(), user_id)
            .change_context(AddUrlServiceError::DbError)?;

        Ok(())
    }
}

impl FromContext for AddUrlService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shorty::form::add_edit_url_form::AddEditUrlForm;
    use crate::shorty::repository::shorty_repository::ShortyRepositoryError;

    #[tokio::test]
    async fn test_add_url_submit_success() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_add_url_redirect("hello", "http://hello.com", 1)
            .returns_once(Ok(()));

        let add_url_service = AddUrlService::new(shorty_repository);

        let mut add_edit_url_form = AddEditUrlForm::default();
        add_edit_url_form.url_path = "hello".to_string();
        add_edit_url_form.url_redirect = "http://hello.com".to_string();

        let validated = add_edit_url_form.as_validated().await.0.unwrap();

        let result = add_url_service.add_url_submit(&validated, 1);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_url_submit_db_error() {
        let mut shorty_repository = ShortyRepository::new_mock();
        shorty_repository
            .mock_add_url_redirect("hello", "http://hello.com", 1)
            .returns_once(Err(Report::new(ShortyRepositoryError::QueryError)));

        let add_url_service = AddUrlService::new(shorty_repository);

        let mut add_edit_url_form = AddEditUrlForm::default();
        add_edit_url_form.url_path = "hello".to_string();
        add_edit_url_form.url_redirect = "http://hello.com".to_string();

        let validated = add_edit_url_form.as_validated().await.0.unwrap();

        let result = add_url_service.add_url_submit(&validated, 1);
        assert!(result.is_err());
    }
}
