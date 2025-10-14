use crate::shorty::model::url::UrlRedirect;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use rusqlite::{Connection, OptionalExtension, named_params};
use shared::context::{Context, ContextError, FromContext};
use shared::db::SqliteClient;
use shared::error::LogItExt;
use std::sync::MutexGuard;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortyRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Row Value error")]
    RowValueError,
    #[error("Lock error")]
    LockError,
}

#[mry::mry]
pub struct ShortyRepository {
    sqlite_client: Option<SqliteClient>,
}

impl ShortyRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self {
            sqlite_client: Some(sqlite_client),
            mry: Default::default(),
        }
    }

    fn borrow_conn(&'_ self) -> Result<MutexGuard<'_, Connection>, Report<ShortyRepositoryError>> {
        let guard = self
            .sqlite_client
            .as_ref()
            .expect("Client")
            .get_conn()
            .lock()
            .map_err(|err| {
                Report::new(ShortyRepositoryError::LockError)
                    .attach(err.to_string())
                    .log_it()
                    .attach(StatusCode::INTERNAL_SERVER_ERROR)
            })?;
        Ok(guard)
    }
}

#[mry::mry]
impl ShortyRepository {
    pub fn fetch_url(
        &self,
        path: &str,
    ) -> Result<Option<UrlRedirect>, Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!("_sql/shorty/fetch_url.sql"))
            .change_context(ShortyRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let row = stmt
            .query_row(
                named_params! {
                    ":path": path,
                },
                |row| {
                    Ok(UrlRedirect {
                        url_redirect: row.get("url_redirect")?,
                    })
                },
            )
            .optional()
            .change_context(ShortyRepositoryError::RowValueError)
            .attach(StatusCode::UNPROCESSABLE_ENTITY)?;

        Ok(row)
    }
}

#[cfg(test)]
impl ShortyRepository {
    pub fn new_mock() -> Self {
        mry::new!(Self {
            sqlite_client: None
        })
    }
}

impl FromContext for ShortyRepository {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
