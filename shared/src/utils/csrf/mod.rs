use crate::utils::error::FromErrorStack;
use crate::utils::query_string::form::FormQs;
use error_stack::{Report, ResultExt};
use maud::{Markup, html};
use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::web::{CsrfToken, CsrfVerifier, Form, Json};
use poem::{Endpoint, FromRequest, IntoResponse, Request, RequestBody, Response, get, handler};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::error::Error;
use std::ops::Deref;
use thiserror::Error;

pub const CSRF_PATH: &'static str = "/csrf/";

pub trait CsrfTokenHtml {
    fn as_html(&self) -> Markup;
}

impl CsrfTokenHtml for CsrfToken {
    fn as_html(&self) -> Markup {
        html! {
            input type="hidden" name="csrf_token" value=(self.0);
        }
    }
}

#[derive(Debug, Error)]
#[error("csrf error")]
pub struct CsrfError;

impl ResponseError for CsrfError {
    fn status(&self) -> StatusCode {
        StatusCode::UNAUTHORIZED
    }

    fn as_response(&self) -> Response
    where
        Self: Error + Send + Sync + 'static,
    {
        ().with_status(self.status()).into_response()
    }
}

pub trait CsrfVerifierError {
    fn verify(&self, token: &str) -> Result<(), Report<CsrfError>>;
}

impl CsrfVerifierError for CsrfVerifier {
    fn verify(&self, token: &str) -> Result<(), Report<CsrfError>> {
        self.validate(token)
            .change_context(CsrfError)
            .attach(StatusCode::UNAUTHORIZED)
    }
}

#[derive(Clone)]
pub struct CsrfHeaderValid;

struct CsrfTokenChecker<E: Endpoint>(E, bool);

impl<E: Endpoint> Endpoint for CsrfTokenChecker<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        let token = req.header("X-Csrf-Token");
        match token {
            None => {
                if self.1 {
                    return Err(CsrfError.into());
                }
                // will check csrf token in payload later
                Ok(self.0.call(req).await?)
            }
            Some(token) => {
                let csrf_verifier = <&CsrfVerifier>::from_request_without_body(&req).await?;
                if csrf_verifier.is_valid(token) {
                    req.set_data(CsrfHeaderValid);
                    Ok(self.0.call(req).await?)
                } else {
                    Err(CsrfError.into())
                }
            }
        }
    }
}

pub fn csrf_header_check<E: Endpoint>(endpoint: E) -> impl Endpoint {
    CsrfTokenChecker(endpoint, false)
}

pub fn csrf_header_check_strict<E: Endpoint>(endpoint: E) -> impl Endpoint {
    CsrfTokenChecker(endpoint, true)
}

#[handler]
async fn fetch_csrf_token(token: &CsrfToken) -> Json<Value> {
    Json(json!({"token": token.0}))
}

pub fn route_csrf() -> poem::Route {
    poem::Route::new().at("/token", get(fetch_csrf_token))
}

#[derive(Deserialize)]
struct CsrfFormBody<T> {
    csrf_token: String,
    #[serde(flatten)]
    data: T,
}

pub struct CsrfForm<T>(pub T);

impl<T> Deref for CsrfForm<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for CsrfForm<T> {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> poem::Result<Self> {
        if req.data::<CsrfHeaderValid>().is_some() {
            let form = Form::<T>::from_request(req, body).await?;
            Ok(Self(form.0))
        } else {
            let csrf_verifier = <&CsrfVerifier>::from_request_without_body(req).await?;
            let form = Form::<CsrfFormBody<T>>::from_request(req, body).await?;
            csrf_verifier
                .verify(form.0.csrf_token.as_str())
                .map_err(poem::Error::from_error_stack)?;
            Ok(Self(form.0.data))
        }
    }
}

pub struct CsrfFormQs<T>(pub T);

impl<T> Deref for CsrfFormQs<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for CsrfFormQs<T> {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> poem::Result<Self> {
        if req.data::<CsrfHeaderValid>().is_some() {
            let form = FormQs::<T>::from_request(req, body).await?;
            Ok(Self(form.0))
        } else {
            let csrf_verifier = <&CsrfVerifier>::from_request_without_body(req).await?;
            let form = FormQs::<CsrfFormBody<T>>::from_request(req, body).await?;
            csrf_verifier
                .verify(form.0.csrf_token.as_str())
                .map_err(poem::Error::from_error_stack)?;
            Ok(Self(form.0.data))
        }
    }
}
