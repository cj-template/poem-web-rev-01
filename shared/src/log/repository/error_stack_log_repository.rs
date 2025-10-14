use crate::context::{Context, ContextError, FromContext};
use crate::db::{BorrowConnectionExt, SqliteClient};
use error_stack::{Report, ResultExt};
use rusqlite::{Connection, named_params};
use std::sync::MutexGuard;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorStackLogRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Borrow Conn error")]
    BorrowConnError,
}

#[mry::mry]
pub struct ErrorStackLogRepository {
    sqlite_client: Option<SqliteClient>,
}

impl ErrorStackLogRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self {
            sqlite_client: Some(sqlite_client),
            mry: Default::default(),
        }
    }

    fn borrow_conn(
        &'_ self,
    ) -> Result<MutexGuard<'_, Connection>, Report<ErrorStackLogRepositoryError>> {
        self.sqlite_client
            .borrow_conn()
            .change_context(ErrorStackLogRepositoryError::BorrowConnError)
    }
}

#[mry::mry]
impl ErrorStackLogRepository {
    pub fn add_to_log(
        &self,
        error_name: &str,
        error_summary: &str,
        error_stack: &str,
    ) -> Result<(), Report<ErrorStackLogRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/error_stack_log_repository/add_to_log.sql"),
            named_params! {
                ":error_name": error_name,
                ":error_summary": error_summary,
                ":error_stack": error_stack,
            },
        )
        .change_context(ErrorStackLogRepositoryError::QueryError)?;

        Ok(())
    }
}

#[cfg(test)]
impl ErrorStackLogRepository {
    pub fn new_mock() -> Self {
        mry::new!(Self {
            sqlite_client: None
        })
    }
}

impl FromContext for ErrorStackLogRepository {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
