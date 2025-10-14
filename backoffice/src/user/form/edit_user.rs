use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::html::validate::ValidateErrorMessageExt;
use crate::user::form::locale::UserFormLocale;
use crate::user::role::Role;
use crate::user::rule::user_manager::UsernameUserManagerRulesExt;
use cjtoolkit_structured_validator::common::flag_error::FlagCounter;
use cjtoolkit_structured_validator::types::username::{
    IsUsernameTakenAsync, Username, UsernameError,
};
use maud::{Markup, html};
use poem::i18n::Locale;
use serde::{Deserialize, Serialize};
use shared::locale::LocaleExtForResult;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub struct EditUserForm {
    pub username: String,
    pub role: Role,
    pub csrf_token: String,
}

impl EditUserForm {
    pub async fn as_validated<T: IsUsernameTakenAsync>(
        &self,
        service: &T,
        current_user_name: &str,
    ) -> EditUserResult {
        EditUserResult(
            async {
                let mut flag = FlagCounter::new();

                let username = flag.check(
                    Username::parse_user_add(
                        Some(&self.username.trim()),
                        service,
                        Some(current_user_name),
                    )
                    .await,
                );

                if flag.is_flagged() {
                    return Err(EditUserError {
                        username,
                        role: self.role.clone(),
                    });
                }

                Ok(EditUserValidated {
                    username: username.expect("Username is not empty"),
                    role: self.role.clone(),
                })
            }
            .await,
        )
    }

    pub async fn as_form_html(
        &self,
        context_html_builder: &ContextHtmlBuilder,
        errors: Option<EditUserMessage>,
        token: Option<Markup>,
        username: Option<String>,
    ) -> Markup {
        let errors = errors.unwrap_or_default();
        let token = token.unwrap_or_default();
        let user_form_locale = UserFormLocale::new(&context_html_builder.locale);
        let username = username.unwrap_or_default();
        context_html_builder.attach_title(&user_form_locale.title_edit).attach_content(html! {
            h1 .mt-3 { (user_form_locale.title_edit) }
            h2 { (username) }
            form hx-boost="true" hx-target="#main-content" .form method="post" {
                (token)
                div .form-group {
                    label .label for="username" { (user_form_locale.username) } br;
                    input .form-item .w-full type="text" name="username" #username value=(self.username)
                    placeholder=(user_form_locale.username_placeholder) {}
                    (errors.username.into_error_html())
                }
                div .form-group {
                    label .label for="role" { (user_form_locale.role) } br;
                    select .form-item .w-full name="role" #role {
                        (self.role.html_option())
                    }
                }
                div .form-group {
                    input .btn .btn-sky-blue type="submit" value=(user_form_locale.submit_edit) {}
                }
            }
        }).build()
    }
}

pub struct EditUserValidated {
    pub username: Username,
    pub role: Role,
}

#[cfg(test)]
impl EditUserValidated {
    pub fn new_test_data() -> Self {
        Self {
            username: Username::parse(Some("username")).expect("test username"),
            role: Default::default(),
        }
    }
}

pub struct EditUserError {
    pub username: Result<Username, UsernameError>,
    #[allow(dead_code)]
    pub role: Role,
}

impl EditUserError {
    pub fn as_message(&self, locale: &Locale) -> EditUserMessage {
        EditUserMessage {
            username: self.username.as_translated_message(locale),
        }
    }
}

pub struct EditUserResult(pub Result<EditUserValidated, EditUserError>);

#[derive(Debug, Clone, Serialize, Default)]
pub struct EditUserMessage {
    pub username: Arc<[String]>,
}
