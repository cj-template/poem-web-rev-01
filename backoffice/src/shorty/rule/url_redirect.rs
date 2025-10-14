use cjtoolkit_structured_validator::types::url::{Url, UrlError, UrlRules};

fn url_redirect_rule() -> UrlRules {
    UrlRules { is_mandatory: true }
}

pub trait UrlRedirectRulesExt {
    fn parse_url_redirect(url_redirect: Option<&str>) -> Result<Url, UrlError>;
}

impl UrlRedirectRulesExt for Url {
    fn parse_url_redirect(url_redirect: Option<&str>) -> Result<Url, UrlError> {
        Self::parse_custom(url_redirect, url_redirect_rule())
    }
}
