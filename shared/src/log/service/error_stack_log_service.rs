use crate::context::{Context, ContextError, FromContext};
use crate::error::LogData;
use crate::log::repository::error_stack_log_repository::ErrorStackLogRepository;
use error_stack::{Report, ResultExt};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Error Stack Log Service Error")]
pub struct ErrorStackLogServiceError;

pub struct ErrorStackLogService {
    error_stack_log_repository: ErrorStackLogRepository,
}

impl ErrorStackLogService {
    pub fn new(error_stack_log_repository: ErrorStackLogRepository) -> Self {
        Self {
            error_stack_log_repository,
        }
    }

    pub fn log_data(&self, log_data: &LogData) -> Result<(), Report<ErrorStackLogServiceError>> {
        self.error_stack_log_repository
            .add_to_log(&log_data.name, &log_data.summary, &log_data.details)
            .change_context(ErrorStackLogServiceError)
    }
}

impl FromContext for ErrorStackLogService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::log::repository::error_stack_log_repository::ErrorStackLogRepositoryError;

    #[test]
    fn test_error_stack_log_service_success() {
        let mut error_stack_log_repository = ErrorStackLogRepository::new_mock();
        let log_data = LogData {
            name: "abc".to_string(),
            summary: "efg".to_string(),
            details: "123".to_string(),
        };
        error_stack_log_repository
            .mock_add_to_log(
                log_data.name.clone(),
                log_data.summary.clone(),
                log_data.details.clone(),
            )
            .returns_once(Ok(()));

        let service = ErrorStackLogService::new(error_stack_log_repository);
        let result = service.log_data(&log_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_error_stack_log_service_error() {
        let mut error_stack_log_repository = ErrorStackLogRepository::new_mock();
        let log_data = LogData {
            name: "abc".to_string(),
            summary: "efg".to_string(),
            details: "123".to_string(),
        };
        error_stack_log_repository
            .mock_add_to_log(
                log_data.name.clone(),
                log_data.summary.clone(),
                log_data.details.clone(),
            )
            .returns_once(Err(Report::new(ErrorStackLogRepositoryError::QueryError)));

        let service = ErrorStackLogService::new(error_stack_log_repository);
        let result = service.log_data(&log_data);
        assert!(result.is_err());
    }
}
