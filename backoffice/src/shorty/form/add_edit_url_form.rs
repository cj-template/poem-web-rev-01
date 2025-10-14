use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::html::validate::ValidateErrorMessageExt;
use crate::shorty::form::locale::ShortyFormLocale;
use crate::shorty::rule::url_path::UrlPathRulesExt;
use crate::shorty::rule::url_redirect::UrlRedirectRulesExt;
use cjtoolkit_structured_validator::common::flag_error::FlagCounter;
use cjtoolkit_structured_validator::types::name::name_alias::{Field, FieldError};
use cjtoolkit_structured_validator::types::url::{Url, UrlError};
use maud::{Markup, html};
use poem::i18n::Locale;
use serde::Deserialize;
use shared::locale::LocaleExtForResult;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub struct AddEditUrlForm {
    pub url_path: String,
    pub url_redirect: String,
    pub csrf_token: String,
}

impl AddEditUrlForm {
    pub async fn as_validated(&self) -> AddEditUrlResult {
        AddEditUrlResult(
            async {
                let mut flag = FlagCounter::new();

                let url_path = flag.check(Field::parse_url_path(Some(&self.url_path.trim())));
                let url_redirect =
                    flag.check(Url::parse_url_redirect(Some(&self.url_redirect.trim())));

                if flag.is_flagged() {
                    return Err(AddEditUrlError {
                        url_path,
                        url_redirect,
                    });
                }

                Ok(AddEditUrlValidated {
                    url_path: url_path.expect("Url path is not empty"),
                    url_redirect: url_redirect.expect("Url redirect is not empty"),
                })
            }
            .await,
        )
    }

    pub async fn as_form_html(
        &self,
        context_html_builder: &ContextHtmlBuilder,
        errors: Option<AddEditUrlMessage>,
        token: Option<Markup>,
        is_edit: bool,
    ) -> Markup {
        let errors = errors.unwrap_or_default();
        let token = token.unwrap_or_default();

        let user_form_locale = ShortyFormLocale::new(&context_html_builder.locale);
        let title = if is_edit {
            &user_form_locale.title_edit
        } else {
            &user_form_locale.title_add
        };

        context_html_builder.attach_title(title).attach_content(html! {
            h1 .mt-3 { (title) }
            form hx-boost="true" hx-target="#main-content" .form method="post" {
                (token)
                div .form-group {
                    label .label for="url-path" { (&user_form_locale.url_path) } br;
                    input .form-item .w-full type="text" name="url_path" #url-path value=(self.url_path)
                    placeholder=(&user_form_locale.url_path_placeholder) {}
                    (errors.url_path.into_error_html())
                }
                div .form-group {
                    label .label for="url-redirect" { (&user_form_locale.url_redirect) } br;
                    input .form-item .w-full type="text" name="url_redirect" #url-redirect value=(self.url_redirect)
                    placeholder=(&user_form_locale.url_redirect_placeholder) {}
                    (errors.url_redirect.into_error_html())
                }
                div .form-group {
                    input .btn .btn-sky-blue type="submit" value=(&user_form_locale.submit_button) {}
                }
            }
        }).build()
    }
}

pub struct AddEditUrlValidated {
    pub url_path: Field,
    pub url_redirect: Url,
}

#[derive(Debug)]
pub struct AddEditUrlError {
    pub url_path: Result<Field, FieldError>,
    pub url_redirect: Result<Url, UrlError>,
}

impl AddEditUrlError {
    pub fn as_message(&self, locale: &Locale) -> AddEditUrlMessage {
        AddEditUrlMessage {
            url_path: self.url_path.as_translated_message(locale),
            url_redirect: self.url_redirect.as_translated_message(locale),
        }
    }
}

pub struct AddEditUrlResult(pub Result<AddEditUrlValidated, AddEditUrlError>);

#[derive(Debug, Default)]
pub struct AddEditUrlMessage {
    pub url_path: Arc<[String]>,
    pub url_redirect: Arc<[String]>,
}
