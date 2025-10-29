use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;

use poem::error::ResponseError;
use poem::http::StatusCode;
use poem::{
    FromRequest, Request, Result,
    http::{
        Method,
        header::{self},
    },
    web::RequestBody,
};

#[derive(Debug, thiserror::Error)]
pub enum ParseFormError {
    /// Invalid content type.
    #[error("invalid content type `{0}`, expect: `application/x-www-form-urlencoded`")]
    InvalidContentType(String),

    /// `Content-Type` header is required.
    #[error("expect content type `application/x-www-form-urlencoded`")]
    ContentTypeRequired,

    /// Url decode error.
    #[error("url decode: {0}")]
    UrlDecode(#[from] serde_qs::Error),
}

impl ResponseError for ParseFormError {
    fn status(&self) -> StatusCode {
        match self {
            Self::InvalidContentType(_) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::ContentTypeRequired => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::UrlDecode(_) => StatusCode::BAD_REQUEST,
        }
    }
}

pub struct FormQs<T>(pub T);

impl<T> Deref for FormQs<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for FormQs<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for FormQs<T> {
    async fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let config = req
            .data::<serde_qs::Config>()
            .map(|v| v.clone())
            .unwrap_or_default();

        if req.method() == Method::GET {
            Ok(config
                .deserialize_str(req.uri().query().unwrap_or_default())
                .map_err(ParseFormError::UrlDecode)
                .map(Self)?)
        } else {
            let content_type = req
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|content_type| content_type.to_str().ok())
                .ok_or(ParseFormError::ContentTypeRequired)?;
            if !is_form_content_type(content_type) {
                return Err(ParseFormError::InvalidContentType(content_type.into()).into());
            }

            Ok(Self(
                config
                    .deserialize_bytes(&body.take()?.into_vec().await?)
                    .map_err(ParseFormError::UrlDecode)?,
            ))
        }
    }
}

fn is_form_content_type(content_type: &str) -> bool {
    matches!(content_type.parse::<mime::Mime>(),
        Ok(content_type) if content_type.type_() == "application"
        && (content_type.subtype() == "x-www-form-urlencoded"
        || content_type
            .suffix()
            .is_some_and(|v| v == "x-www-form-urlencoded")))
}
