pub mod response;

use crate::context::{Context, ContextError, FromContext};
use crate::htmx::response::{HtmxResponse, HtmxResponseExt};
use error_stack::{Report, ResultExt};
use poem::http::{StatusCode, header};
use poem::web::Redirect;
use poem::{Endpoint, FromRequest, IntoResponse, Request, RequestBody, Response};
use serde_json::json;
use std::ops::Deref;
use std::sync::Arc;

pub struct HtmxHeaderData {
    pub boosted: bool,
    pub current_url: Option<String>,
    pub history_restore_request: bool,
    pub prompt: Option<String>,
    pub request: bool,
    pub target: Option<String>,
    pub trigger_name: Option<String>,
    pub trigger: Option<String>,
}

impl HtmxHeaderData {
    fn new(req: &Request) -> Self {
        let headers = req.headers();
        Self {
            boosted: headers
                .get("HX-Boosted")
                .map(|s| s.to_str().unwrap_or_default() == "true")
                .unwrap_or_default(),
            current_url: headers
                .get("HX-Current-URL")
                .map(|s| s.to_str().unwrap_or_default().to_string()),
            history_restore_request: headers
                .get("HX-History-Restore-Request")
                .map(|s| s.to_str().unwrap_or_default() == "true")
                .unwrap_or_default(),
            prompt: headers
                .get("HX-Prompt")
                .map(|s| s.to_str().unwrap_or_default().to_string()),
            request: headers.get("HX-Request").is_some(),
            target: headers
                .get("HX-Target")
                .map(|s| s.to_str().unwrap_or_default().to_string()),
            trigger_name: headers
                .get("HX-Trigger-Name")
                .map(|s| s.to_str().unwrap_or_default().to_string()),
            trigger: headers
                .get("HX-Trigger")
                .map(|s| s.to_str().unwrap_or_default().to_string()),
        }
    }
}

pub struct HtmxHeader(Arc<HtmxHeaderData>);

impl Deref for HtmxHeader {
    type Target = HtmxHeaderData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Clone for HtmxHeader {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub async fn htmx_request_around<EP: Endpoint>(
    next: EP,
    mut req: Request,
) -> poem::Result<EP::Output> {
    let header = HtmxHeader(Arc::new(HtmxHeaderData::new(&req)));
    req.set_data(header);
    next.call(req).await
}

impl<'a> FromRequest<'a> for HtmxHeader {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let header = req.data::<HtmxHeader>().ok_or_else(|| {
            poem::Error::from_string("missing header", StatusCode::INTERNAL_SERVER_ERROR)
        })?;
        Ok(header.clone())
    }
}

impl FromContext for HtmxHeader {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let req = ctx.req_result()?;
        let header = Self::from_request_without_body(req)
            .await
            .change_context(ContextError::RequestError)?;
        Ok(header)
    }
}

impl HtmxHeader {
    pub fn do_location_htmx_response(&self, redirect: Redirect, target: &str) -> HtmxResponse {
        let redirect = redirect.into_response();
        let redirect_url = redirect.header(header::LOCATION).unwrap_or_default();
        ().htmx_response().location(
            json!({"path": redirect_url, "target": target})
                .to_string()
                .as_str(),
        )
    }

    pub fn do_location(&self, redirect: Redirect, target: &str) -> Response {
        if self.request {
            return self
                .do_location_htmx_response(redirect, target)
                .into_response();
        }
        redirect.into_response()
    }

    pub fn do_redirect_htmx_response(&self, redirect: Redirect) -> HtmxResponse {
        let redirect = redirect.into_response();
        let redirect_url = redirect.header(header::LOCATION).unwrap_or_default();
        ().htmx_response().redirect(redirect_url)
    }

    pub fn do_redirect(&self, redirect: Redirect) -> Response {
        if self.request {
            return self.do_redirect_htmx_response(redirect).into_response();
        }
        redirect.into_response()
    }

    pub fn do_htmx_response(&self, htmx_response: HtmxResponse) -> Response {
        if self.request {
            return htmx_response.into_response();
        }
        htmx_response.response
    }
}
