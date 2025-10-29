use poem::http::HeaderValue;
use poem::{IntoResponse, Response};
use std::sync::RwLock;

#[derive(Default)]
struct HtmxResponseHeader {
    location: Option<String>,
    push_url: Option<String>,
    redirect: Option<String>,
    refresh: bool,
    replace_url: Option<String>,
    reswap: Option<String>,
    retarget: Option<String>,
    reselect: Option<String>,
    trigger: Option<String>,
    trigger_after_settle: Option<String>,
    trigger_after_swap: Option<String>,
}

#[derive(Default)]
pub struct HtmxResponse {
    pub response: Response,
    header: RwLock<HtmxResponseHeader>,
}

impl HtmxResponse {
    pub fn location(self, location: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.location = Some(location.to_string());
        }
        self
    }

    pub fn push_url(self, push_url: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.push_url = Some(push_url.to_string());
        }
        self
    }

    pub fn redirect(self, redirect: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.redirect = Some(redirect.to_string());
        }
        self
    }

    pub fn refresh(self) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.refresh = true;
        }
        self
    }

    pub fn replace_url(self, replace_url: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.replace_url = Some(replace_url.to_string());
        }
        self
    }

    pub fn reswap(self, reswap: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.reswap = Some(reswap.to_string());
        }
        self
    }

    pub fn retarget(self, retarget: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.retarget = Some(retarget.to_string());
        }
        self
    }

    pub fn reselect(self, reselect: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.reselect = Some(reselect.to_string());
        }
        self
    }

    pub fn trigger(self, trigger: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.trigger = Some(trigger.to_string());
        }
        self
    }

    pub fn trigger_after_settle(self, trigger_after_settle: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.trigger_after_settle = Some(trigger_after_settle.to_string());
        }
        self
    }

    pub fn trigger_after_swap(self, trigger_after_swap: &str) -> Self {
        if let Ok(mut header) = self.header.try_write() {
            header.trigger_after_swap = Some(trigger_after_swap.to_string());
        }
        self
    }
}

pub trait HtmxResponseExt {
    fn htmx_response(self) -> HtmxResponse;
}

impl<T: IntoResponse> HtmxResponseExt for T {
    fn htmx_response(self) -> HtmxResponse {
        let mut htmx_response = HtmxResponse::default();
        htmx_response.response = self.into_response();
        htmx_response
    }
}

impl IntoResponse for HtmxResponse {
    fn into_response(self) -> Response {
        let mut res = self.response;
        if let Ok(header) = self.header.try_read() {
            let header_map = res.headers_mut();
            if let Some(location) = &header.location
                && let Some(location) = HeaderValue::from_str(&location).ok()
            {
                header_map.insert("HX-Location", location);
            }
            if let Some(push_url) = &header.push_url
                && let Some(push_url) = HeaderValue::from_str(&push_url).ok()
            {
                header_map.insert("HX-Push-Url", push_url);
            }
            if let Some(redirect) = &header.redirect
                && let Some(redirect) = HeaderValue::from_str(&redirect).ok()
            {
                header_map.insert("HX-Redirect", redirect);
            }
            if header.refresh {
                header_map.insert("HX-Refresh", HeaderValue::from_static("true"));
            }
            if let Some(replace_url) = &header.replace_url
                && let Some(replace_url) = HeaderValue::from_str(&replace_url).ok()
            {
                header_map.insert("HX-Replace-Url", replace_url);
            }
            if let Some(reswap) = &header.reswap
                && let Some(reswap) = HeaderValue::from_str(&reswap).ok()
            {
                header_map.insert("HX-Reswap", reswap);
            }
            if let Some(retarget) = &header.retarget
                && let Some(retarget) = HeaderValue::from_str(&retarget).ok()
            {
                header_map.insert("HX-Retarget", retarget);
            }
            if let Some(reselect) = &header.reselect
                && let Some(reselect) = HeaderValue::from_str(&reselect).ok()
            {
                header_map.insert("HX-Reselect", reselect);
            }
            if let Some(trigger) = &header.trigger
                && let Some(trigger) = HeaderValue::from_str(&trigger).ok()
            {
                header_map.insert("HX-Trigger", trigger);
            }
            if let Some(trigger_after_settle) = &header.trigger_after_settle
                && let Some(trigger_after_settle) =
                    HeaderValue::from_str(&trigger_after_settle).ok()
            {
                header_map.insert("HX-Trigger-After-Settle", trigger_after_settle);
            }
            if let Some(trigger_after_swap) = &header.trigger_after_swap
                && let Some(trigger_after_swap) = HeaderValue::from_str(&trigger_after_swap).ok()
            {
                header_map.insert("HX-Trigger-After-Swap", trigger_after_swap);
            }
        }

        res
    }
}
