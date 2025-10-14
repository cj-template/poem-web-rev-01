use crate::stack::model::stack_model::{ListStackModel, StackModel};
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use rusqlite::{Connection, OptionalExtension, named_params};
use shared::context::{Context, ContextError, FromContext};
use shared::db::{BorrowConnectionExt, SqliteClient};
use std::sync::{Arc, MutexGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StackRepositoryError {
    #[error("Query Error")]
    QueryError,
    #[error("Row Value Error")]
    RowValueError,
    #[error("Borrow Conn Error")]
    BorrowConnError,
}

#[mry::mry]
pub struct StackRepository {
    sqlite_client: Option<SqliteClient>,
}

impl StackRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self {
            sqlite_client: Some(sqlite_client),
            mry: Default::default(),
        }
    }

    fn borrow_conn(&'_ self) -> Result<MutexGuard<'_, Connection>, Report<StackRepositoryError>> {
        self.sqlite_client
            .borrow_conn()
            .change_context(StackRepositoryError::BorrowConnError)
    }
}

#[mry::mry]
impl StackRepository {
    pub fn clear(&self) -> Result<(), Report<StackRepositoryError>> {
        let conn = self.borrow_conn()?;
        conn.execute(
            include_str!("_sql/stack_repository/clear.sql"),
            named_params! {},
        )
        .change_context(StackRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub fn fetch_error_stack(
        &self,
        id: i64,
    ) -> Result<Option<StackModel>, Report<StackRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare(include_str!("_sql/stack_repository/fetch_error_stack.sql"))
            .change_context(StackRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let row = stmt
            .query_one(
                named_params! {
                    ":id": id
                },
                |row| {
                    Ok(StackModel {
                        id: row.get("id")?,
                        error_name: row.get("error_name")?,
                        error_summary: row.get("error_summary")?,
                        error_stack: row.get("error_stack")?,
                        reported_at: row.get("reported_at")?,
                    })
                },
            )
            .optional()
            .change_context(StackRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(row)
    }

    pub fn list_error_stack(&self) -> Result<Arc<[ListStackModel]>, Report<StackRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare(include_str!("_sql/stack_repository/list_error_stack.sql"))
            .change_context(StackRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let rows_iter = stmt
            .query_map(named_params! {}, |row| {
                Ok(ListStackModel {
                    id: row.get("id")?,
                    error_name: row.get("error_name")?,
                    error_summary: row.get("error_summary")?,
                    reported_at: row.get("reported_at")?,
                })
            })
            .change_context(StackRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let items = rows_iter
            .collect::<Result<Vec<_>, _>>()
            .change_context(StackRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(items.into())
    }
}

#[cfg(test)]
impl StackRepository {
    pub fn new_mock() -> Self {
        mry::new!(Self {
            sqlite_client: None
        })
    }
}

impl FromContext for StackRepository {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
