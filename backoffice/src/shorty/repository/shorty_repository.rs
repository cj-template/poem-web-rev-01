use crate::shorty::model::shorty_model::{
    GetUrlRedirectModel, GetUserIdByUrlIdModel, ListUrlRedirectModel,
};
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use rusqlite::{Connection, OptionalExtension, named_params};
use shared::context::{Context, ContextError, FromContext};
use shared::db::{BorrowConnectionExt, SqliteClient};
use std::sync::{Arc, MutexGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShortyRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Row Value error")]
    RowValueError,
    #[error("Borrow Conn error")]
    BorrowConnError,
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
        self.sqlite_client
            .borrow_conn()
            .change_context(ShortyRepositoryError::BorrowConnError)
    }
}

#[mry::mry]
impl ShortyRepository {
    pub fn add_url_redirect(
        &self,
        url_path: &str,
        url_redirect: &str,
        user_id: i64,
    ) -> Result<(), Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/shorty_repository/add_url_redirect.sql"),
            named_params! {
                ":url_path": url_path,
                ":url_redirect": url_redirect,
                ":user_id": user_id,
            },
        )
        .change_context(ShortyRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub fn delete_url_redirect(&self, id: i64) -> Result<(), Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/shorty_repository/delete_url_redirect.sql"),
            named_params! {
                ":id": id,
            },
        )
        .change_context(ShortyRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub fn edit_url_redirect(
        &self,
        id: i64,
        url_path: &str,
        url_redirect: &str,
    ) -> Result<(), Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/shorty_repository/edit_url_redirect.sql"),
            named_params! {
                ":id": id,
                ":url_path": url_path,
                ":url_redirect": url_redirect,
            },
        )
        .change_context(ShortyRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(())
    }

    pub fn get_url_redirect(
        &self,
        id: i64,
    ) -> Result<Option<GetUrlRedirectModel>, Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!("_sql/shorty_repository/get_url_redirect.sql"))
            .map_err(|_| Report::new(ShortyRepositoryError::QueryError))
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let item = stmt
            .query_one(
                named_params! {
                    ":id": id,
                },
                |row| {
                    Ok(GetUrlRedirectModel {
                        url_path: row.get("url_path")?,
                        url_redirect: row.get("url_redirect")?,
                    })
                },
            )
            .optional()
            .change_context(ShortyRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(item)
    }

    pub fn get_user_id_by_url_id(
        &self,
        id: i64,
    ) -> Result<Option<GetUserIdByUrlIdModel>, Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!(
                "_sql/shorty_repository/get_user_id_by_url_id.sql"
            ))
            .change_context(ShortyRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let item = stmt
            .query_one(
                named_params! {
                    ":id": id,
                },
                |row| {
                    Ok(GetUserIdByUrlIdModel {
                        created_by_user_id: row.get("created_by_user_id")?,
                    })
                },
            )
            .optional()
            .change_context(ShortyRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(item)
    }

    pub fn list_url_redirect(
        &self,
    ) -> Result<Arc<[ListUrlRedirectModel]>, Report<ShortyRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!("_sql/shorty_repository/list_url_redirect.sql"))
            .map_err(|_| Report::new(ShortyRepositoryError::QueryError))
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let items_iter = stmt
            .query_map(named_params! {}, |row| {
                Ok(ListUrlRedirectModel {
                    id: row.get("id")?,
                    url_path: row.get("url_path")?,
                    url_redirect: row.get("url_redirect")?,
                    created_at: row.get("created_at")?,
                    created_by_user_id: row.get("created_by_user_id")?,
                    username: row.get("username")?,
                })
            })
            .change_context(ShortyRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let items = items_iter
            .collect::<Result<Vec<_>, _>>()
            .change_context(ShortyRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(items.into())
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
