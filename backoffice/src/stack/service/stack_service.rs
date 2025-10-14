use crate::stack::model::stack_model::{ListStackModel, StackModel};
use crate::stack::repository::stack_repository::StackRepository;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};
use shared::error::ExtraResultExt;
use std::sync::Arc;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StackServiceError {
    #[error("DB error")]
    DbError,
    #[error("Not found")]
    NotFound,
}

pub struct StackService {
    stack_repository: StackRepository,
}

impl StackService {
    pub fn new(stack_repository: StackRepository) -> Self {
        Self { stack_repository }
    }

    pub fn clear(&self) -> Result<(), Report<StackServiceError>> {
        self.stack_repository
            .clear()
            .change_context(StackServiceError::DbError)
            .log_it()
    }

    pub fn fetch_error_stack(&self, id: i64) -> Result<StackModel, Report<StackServiceError>> {
        self.stack_repository
            .fetch_error_stack(id)
            .change_context(StackServiceError::DbError)
            .log_it()?
            .ok_or_else(|| Report::new(StackServiceError::NotFound).attach(StatusCode::NOT_FOUND))
    }

    pub fn list_error_stack(&self) -> Arc<[ListStackModel]> {
        self.stack_repository.list_error_stack().unwrap_or_default()
    }
}

impl FromContext for StackService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stack::repository::stack_repository::StackRepositoryError;

    #[test]
    fn test_stack_service_clear_success() {
        let mut stack_repository = StackRepository::new_mock();
        stack_repository.mock_clear().returns_once(Ok(()));

        let stack_service = StackService::new(stack_repository);
        let result = stack_service.clear();
        assert!(result.is_ok());
    }

    #[test]
    fn test_stack_service_clear_failure() {
        let mut stack_repository = StackRepository::new_mock();
        stack_repository
            .mock_clear()
            .returns_once(Err(Report::new(StackRepositoryError::QueryError)));

        let stack_service = StackService::new(stack_repository);
        let result = stack_service.clear();
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_service_fetch_error_stack_success() {
        let mut stack_repository = StackRepository::new_mock();
        stack_repository
            .mock_fetch_error_stack(1)
            .returns_once(Ok(Some(StackModel {
                id: 1,
                error_name: "1".to_string(),
                error_summary: "1".to_string(),
                error_stack: "1".to_string(),
                reported_at: Default::default(),
            })));

        let stack_service = StackService::new(stack_repository);
        let result = stack_service.fetch_error_stack(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_stack_service_fetch_error_stack_error() {
        let mut stack_repository = StackRepository::new_mock();
        stack_repository
            .mock_fetch_error_stack(1)
            .returns_once(Err(Report::new(StackRepositoryError::QueryError)));

        let stack_service = StackService::new(stack_repository);
        let result = stack_service.fetch_error_stack(1);
        assert!(result.is_err());
    }

    #[test]
    fn test_stack_service_fetch_error_stack_not_found() {
        let mut stack_repository = StackRepository::new_mock();
        stack_repository
            .mock_fetch_error_stack(1)
            .returns_once(Ok(None));

        let stack_service = StackService::new(stack_repository);
        let result = stack_service.fetch_error_stack(1);
        assert!(result.is_err());
        let result = result.err().unwrap();
        let error_code = result.downcast_ref::<StatusCode>().unwrap();
        assert_eq!(error_code, &StatusCode::NOT_FOUND);
    }
}
