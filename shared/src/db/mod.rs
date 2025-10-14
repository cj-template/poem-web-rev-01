use crate::config::ConfigPointer;
use crate::context::{Context, ContextError, FromContext};
use crate::error::{ExtraResultExt, FromIntoStackError, LogItExt};
use crate::password::Password;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use rusqlite::{Connection, named_params};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, MutexGuard};
use thiserror::Error;
use tokio::sync::OnceCell;

pub trait ConnectionMarker: Send + Sync {}

pub struct DefaultConnection;

impl ConnectionMarker for DefaultConnection {}

#[derive(Error, Debug)]
pub enum SqliteClientError {
    #[error("Sqlite file empty")]
    SqliteFileEmpty,
    #[error("Connection error")]
    Connection,
    #[error("Init failed")]
    InitFailed,
    #[error("Connection Option Empty error")]
    OptionEmpty,
    #[error("Lock error: {0}")]
    LockError(String),
}

impl FromIntoStackError for SqliteClientError {}

pub struct SqliteClient<T = DefaultConnection>(Arc<Mutex<Connection>>, PhantomData<T>)
where
    T: ConnectionMarker;

impl<T: ConnectionMarker> SqliteClient<T> {
    pub fn new(sqlite_path: String) -> Result<Self, Report<SqliteClientError>> {
        if sqlite_path.is_empty() {
            return Err(SqliteClientError::SqliteFileEmpty
                .into_stack_error_critical("Sqlite file path is empty".to_string()));
        }
        let file_exist = std::fs::metadata(&sqlite_path).is_ok();

        let conn = Connection::open(sqlite_path)
            .change_context(SqliteClientError::Connection)
            .attach_critical("Sqlite Connection failed".to_string())?;
        if !file_exist {
            conn.execute_batch(include_str!("_sql/init.sql"))
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Init failed".to_string())?;

            let password = Password::hash_password("banana".to_string())
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Failed to hash password".to_string())?
                .encode_to_msg_pack()
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Failed to encode password".to_string())?;

            conn.execute(
                include_str!("_sql/add_user.sql",),
                named_params! {
                    ":username": "admin",
                    ":password": password.to_vec(),
                },
            )
            .change_context(SqliteClientError::InitFailed)
            .attach_critical("Failed to create default user".to_string())?;
        }

        Ok(SqliteClient(Arc::new(Mutex::new(conn)), PhantomData))
    }

    pub fn get_conn(&self) -> &Mutex<Connection> {
        self.0.as_ref()
    }
}

impl<T: ConnectionMarker> Clone for SqliteClient<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), PhantomData)
    }
}

static SQLITE_CLIENT_CACHE: OnceCell<SqliteClient> = OnceCell::const_new();

impl FromContext for SqliteClient {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let sqlite_client: Result<&Self, Report<ContextError>> = SQLITE_CLIENT_CACHE
            .get_or_try_init(|| async {
                let config: ConfigPointer = ctx.inject().await?;
                Ok(Self::new(config.sqlite.path.clone()).change_context(ContextError::Other)?)
            })
            .await;
        Ok(sqlite_client?.clone())
    }
}

pub trait BorrowConnectionExt {
    fn borrow_conn(&'_ self) -> Result<MutexGuard<'_, Connection>, Report<SqliteClientError>>;
}

impl<T: ConnectionMarker> BorrowConnectionExt for SqliteClient<T> {
    fn borrow_conn(&'_ self) -> Result<MutexGuard<'_, Connection>, Report<SqliteClientError>> {
        self.0.lock().map_err(|err| {
            Report::new(SqliteClientError::LockError(err.to_string()))
                .attach(StatusCode::INTERNAL_SERVER_ERROR)
                .log_it()
        })
    }
}

impl<T: ConnectionMarker> BorrowConnectionExt for Option<SqliteClient<T>> {
    fn borrow_conn(&'_ self) -> Result<MutexGuard<'_, Connection>, Report<SqliteClientError>> {
        self.as_ref()
            .ok_or_else(|| {
                Report::new(SqliteClientError::OptionEmpty)
                    .attach(StatusCode::INTERNAL_SERVER_ERROR)
                    .log_it()
            })?
            .borrow_conn()
    }
}
