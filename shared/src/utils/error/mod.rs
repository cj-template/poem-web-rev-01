pub mod boot_error;

use error_stack::{Report, ResultExt};
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::web::Json;
use poem::{IntoResponse, Response};
use serde_json::json;
use std::error::Error;
use std::fmt::{Debug, Display};
use thiserror::Error;

pub trait FromIntoStackError: Error + Sized + Send + Sync + 'static {
    fn from_error_stack<C>(err: &Report<C>) -> Option<&Self> {
        err.downcast_ref::<Self>()
    }

    fn is_in_error_stack<C>(err: &Report<C>) -> bool {
        Self::from_error_stack(err).is_some()
    }

    fn into_stack_error(self) -> Report<Self> {
        Report::new(self)
    }

    fn into_stack_error_critical(self, msg: String) -> Report<Self> {
        Report::new(self).attach(CriticalError(msg))
    }

    fn into_stack_error_as_attachment<E>(self, err: E) -> Report<E>
    where
        E: Error + Sized + Send + Sync + 'static,
    {
        Report::new(err).attach(self)
    }
}

#[derive(Error, Debug)]
#[error("Critical error: {0}")]
pub struct CriticalError(pub String);

impl FromIntoStackError for CriticalError {}

pub fn check_is_critical_error<C>(err: Report<C>) -> Result<Report<C>, Report<C>> {
    if CriticalError::is_in_error_stack::<C>(&err) {
        return Err(err);
    }
    Ok(err)
}

pub fn setup_critical_error_debug_hook() {
    Report::install_debug_hook::<CriticalError>(|value, context| {
        context.push_body(format!("Critical Error: {}", value.0))
    });
}

struct LogIt;

pub trait ExtraResultExt: ResultExt {
    fn attach_critical(self, msg: String) -> Result<Self::Ok, Report<Self::Context>>;

    fn log_it(self) -> Result<Self::Ok, Report<Self::Context>>;

    fn attach_critical_lazy<F>(self, msg: F) -> Result<Self::Ok, Report<Self::Context>>
    where
        F: FnOnce() -> String;

    fn change_context_attach_previous_msg<C>(self, context: C) -> Result<Self::Ok, Report<C>>
    where
        C: Error + Send + Sync + 'static;

    fn change_context_attach_previous_msg_lazy<C, F>(
        self,
        context: F,
    ) -> Result<Self::Ok, Report<C>>
    where
        C: Error + Send + Sync + 'static,
        F: FnOnce() -> C;

    fn change_context_pass_ref_lazy<C, F>(self, context: F) -> Result<Self::Ok, Report<C>>
    where
        C: Error + Send + Sync + 'static,
        F: FnOnce(&Report<Self::Context>) -> C;
}

impl<T, C> ExtraResultExt for Result<T, Report<C>>
where
    C: Error + Send + Sync + 'static,
{
    fn attach_critical(self, msg: String) -> Self {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => Err(report.attach(CriticalError(msg))),
        }
    }

    fn log_it(self) -> Self {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => Err(report.attach_opaque(LogIt)),
        }
    }

    fn attach_critical_lazy<F>(self, msg: F) -> Self
    where
        F: FnOnce() -> String,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => Err(report.attach(CriticalError(msg()))),
        }
    }

    fn change_context_attach_previous_msg<C2>(self, context: C2) -> Result<T, Report<C2>>
    where
        C2: Error + Send + Sync + 'static,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => {
                let msg = report.to_string();
                Err(report.change_context(context).attach(msg))
            }
        }
    }

    fn change_context_attach_previous_msg_lazy<C2, F>(self, context: F) -> Result<T, Report<C2>>
    where
        C2: Error + Send + Sync + 'static,
        F: FnOnce() -> C2,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => {
                let msg = report.to_string();
                Err(report.change_context(context()).attach(msg))
            }
        }
    }

    fn change_context_pass_ref_lazy<C2, F>(self, context: F) -> Result<T, Report<C2>>
    where
        C2: Error + Send + Sync + 'static,
        F: FnOnce(&Report<Self::Context>) -> C2,
    {
        match self {
            Ok(ok) => Ok(ok),
            Err(report) => {
                let context = context(&report);
                Err(report.change_context(context))
            }
        }
    }
}

pub trait LogItExt {
    fn log_it(self) -> Self;
}

impl<T> LogItExt for Report<T> {
    fn log_it(self) -> Self {
        self.attach_opaque(LogIt)
    }
}

pub struct ErrorStackUseJson;

#[derive(Clone)]
pub struct LogData {
    pub name: String,
    pub summary: String,
    pub details: String,
}

struct ErrorStack<T>(Report<T>);

impl<T> Debug for ErrorStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T> Display for ErrorStack<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> Error for ErrorStack<T> {}

impl<T> ResponseError for ErrorStack<T> {
    fn status(&self) -> StatusCode {
        match self.0.downcast_ref::<StatusCode>() {
            Some(status) => *status,
            None => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn as_response(&self) -> Response
    where
        Self: Error + Send + Sync + 'static,
    {
        let status = self.status();
        let body = if cfg!(debug_assertions) {
            format!("{}\n{:?}", status, self.0)
        } else {
            format!("{}\n{}", status, self.0)
        };
        match self.0.downcast_ref::<ErrorStackUseJson>() {
            Some(_) => {
                let json = Json(json!({"msg": body}));
                let mut resp = json.into_response();
                resp.set_status(status);
                resp
            }
            None => {
                let mut resp = body.into_response();
                resp.set_status(status);
                resp
            }
        }
    }
}

pub trait FromErrorStack: Sized {
    fn from_error_stack<T>(err: Report<T>) -> Self
    where
        T: Send + Sync + 'static;
}

impl FromErrorStack for poem::Error {
    fn from_error_stack<T>(err: Report<T>) -> Self
    where
        T: Send + Sync + 'static,
    {
        match err.downcast_ref::<LogIt>() {
            None => Self::from(ErrorStack(err)),
            Some(_) => {
                let data = LogData {
                    name: format!("{}", err),
                    summary: format!("{:#}", err),
                    details: format!("{:?}", err),
                };
                let mut error = Self::from(ErrorStack(err));
                error.set_data(data);
                error
            }
        }
    }
}
