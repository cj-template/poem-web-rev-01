use crate::user::model::user_manager_model::{FetchPassword, FetchUser, ListUser};
use crate::user::role::Role;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use rusqlite::{Connection, OptionalExtension, named_params};
use shared::context::{Context, ContextError, FromContext};
use shared::db::{BorrowConnectionExt, SqliteClient};
use std::sync::{Arc, MutexGuard};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserManagerRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Row Value error")]
    RowValueError,
    #[error("Borrow Conn error")]
    BorrowConnError,
}

#[mry::mry]
pub struct UserManagerRepository {
    sqlite_client: Option<SqliteClient>,
}

impl UserManagerRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self {
            sqlite_client: Some(sqlite_client),
            mry: Default::default(),
        }
    }

    fn borrow_conn(
        &'_ self,
    ) -> Result<MutexGuard<'_, Connection>, Report<UserManagerRepositoryError>> {
        self.sqlite_client
            .borrow_conn()
            .change_context(UserManagerRepositoryError::BorrowConnError)
    }
}

#[mry::mry]
impl UserManagerRepository {
    pub fn add_user(
        &self,
        username: String,
        password: Box<[u8]>,
        role: &Role,
    ) -> Result<(), Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/user_manager_repository/add_user.sql"),
            named_params! {
                ":username": username,
                ":password": password,
                ":role": role.as_stringed(),
            },
        )
        .change_context(UserManagerRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub fn edit_password(
        &self,
        id: i64,
        password: Box<[u8]>,
    ) -> Result<(), Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/user_manager_repository/edit_password.sql"),
            named_params! {
                ":id": id,
                ":password": password,
            },
        )
        .change_context(UserManagerRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub fn edit_user(
        &self,
        id: i64,
        username: String,
        role: &Role,
    ) -> Result<(), Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;
        conn.execute(
            include_str!("_sql/user_manager_repository/edit_user.sql"),
            named_params! {
                ":id": id,
                ":username": username,
                ":role": role.as_stringed(),
            },
        )
        .change_context(UserManagerRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub fn fetch_user(
        &self,
        id: i64,
    ) -> Result<Option<FetchUser>, Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;
        let mut stmt = conn
            .prepare_cached(include_str!("_sql/user_manager_repository/fetch_user.sql"))
            .change_context(UserManagerRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        let row: Option<FetchUser> = stmt
            .query_one(
                named_params! {
                    ":id": id
                },
                |row| {
                    Ok(FetchUser {
                        username: row.get("username")?,
                        role: Role::try_from(row.get::<_, String>("role")?.as_str())
                            .unwrap_or_default(),
                    })
                },
            )
            .optional()
            .change_context(UserManagerRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(row)
    }

    pub fn list_users(&self) -> Result<Arc<[ListUser]>, Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;
        let mut stmt = conn
            .prepare_cached(include_str!("_sql/user_manager_repository/list_users.sql"))
            .change_context(UserManagerRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        let rows = stmt
            .query_map(named_params! {}, |row| {
                Ok(ListUser {
                    id: row.get("id")?,
                    username: row.get("username")?,
                    role: Role::try_from(row.get::<_, String>("role")?.as_str())
                        .unwrap_or_default(),
                })
            })
            .change_context(UserManagerRepositoryError::RowValueError)?;

        let users = rows
            .collect::<Result<Vec<_>, _>>()
            .change_context(UserManagerRepositoryError::RowValueError)?;

        Ok(users.into())
    }

    pub fn revoke_all_token_by_id(
        &self,
        user_id: i64,
    ) -> Result<(), Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;

        conn.execute(
            include_str!("_sql/user_manager_repository/revoke_all_token_by_id.sql"),
            named_params! {
                ":user_id": user_id,
            },
        )
        .change_context(UserManagerRepositoryError::QueryError)
        .attach(StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(())
    }

    pub fn username_taken(
        &self,
        username: String,
    ) -> Result<bool, Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!(
                "_sql/user_manager_repository/username_taken.sql"
            ))
            .change_context(UserManagerRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let row: Option<bool> = stmt
            .query_one(
                named_params! {
                    ":username": username
                },
                |row| Ok(row.get("taken")?),
            )
            .optional()
            .change_context(UserManagerRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(row.unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn fetch_password(
        &self,
        user_id: i64,
    ) -> Result<FetchPassword, Report<UserManagerRepositoryError>> {
        let conn = self.borrow_conn()?;

        let mut stmt = conn
            .prepare_cached(include_str!(
                "_sql/user_manager_repository/fetch_password.sql"
            ))
            .change_context(UserManagerRepositoryError::QueryError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        let row = stmt
            .query_one(
                named_params! {
                    ":id": user_id
                },
                |row| {
                    Ok(FetchPassword {
                        password: row.get("password")?,
                    })
                },
            )
            .change_context(UserManagerRepositoryError::RowValueError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)?;

        Ok(row)
    }
}

#[cfg(test)]
impl UserManagerRepository {
    pub fn new_mock() -> Self {
        mry::new!(Self {
            sqlite_client: None
        })
    }
}

impl FromContext for UserManagerRepository {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
