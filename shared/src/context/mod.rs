use crate::error::FromErrorStack;
use error_stack::Report;
use poem::{FromRequest, Request, RequestBody};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Config error")]
    ConfigError,
    #[error("Request error")]
    RequestError,
    #[error("Other error")]
    Other,
}

pub trait FromContext: Sized + Send + Sync {
    fn from_context(
        ctx: &'_ Context,
    ) -> impl Future<Output = Result<Self, Report<ContextError>>> + Send;
}

pub struct Context<'a> {
    req: Option<&'a Request>,
}

impl Context<'_> {
    pub async fn inject<T: FromContext>(&self) -> Result<T, Report<ContextError>> {
        T::from_context(self).await
    }

    pub async fn inject_poem<T: FromContext>(&self) -> poem::Result<T> {
        self.inject::<T>()
            .await
            .map_err(poem::Error::from_error_stack)
    }

    pub fn req_result(&self) -> Result<&Request, Report<ContextError>> {
        self.req
            .ok_or_else(|| Report::new(ContextError::RequestError))
    }
}

impl<'a> FromRequest<'a> for Context<'a> {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        Ok(Self { req: Some(req) })
    }
}

pub struct Dep<T: FromContext>(pub T);

impl<'a, T: FromContext> FromRequest<'a> for Dep<T> {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let context = Box::pin(Context { req: Some(req) });
        Ok(Self(
            T::from_context(&context)
                .await
                .map_err(poem::Error::from_error_stack)?,
        ))
    }
}

impl<T: FromContext> Dep<T> {
    pub async fn without_request() -> Result<T, Report<ContextError>> {
        T::from_context(&Context { req: None }).await
    }
}

pub async fn fetch_context<T: FromContext>() -> Result<T, Report<ContextError>> {
    Dep::<T>::without_request().await
}
