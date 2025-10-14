use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::html::validate::ValidateErrorMessageExt;
use crate::user::form::locale::UserFormLocale;
use crate::user::rule::user_manager::PasswordUserManagerRulesExt;
use cjtoolkit_structured_validator::common::flag_error::FlagCounter;
use cjtoolkit_structured_validator::types::password::{Password, PasswordError};
use maud::{Markup, html};
use poem::i18n::Locale;
use serde::{Deserialize, Serialize};
use shared::locale::LocaleExtForResult;
use std::sync::Arc;

#[derive(Deserialize, Default)]
pub struct EditPasswordManagerForm {
    pub password: String,
    pub password_confirm: String,
    pub csrf_token: String,
}

impl EditPasswordManagerForm {
    pub async fn as_validated(&self) -> EditPasswordManagerResult {
        EditPasswordManagerResult(
            async {
                let mut flag = FlagCounter::new();

                let (password, password_confirm) = Password::parse_password_add(
                    Some(&self.password.trim()),
                    &self.password_confirm.trim(),
                );
                let password = flag.check(password);
                let password_confirm = flag.check(password_confirm);

                if flag.is_flagged() {
                    return Err(EditPasswordManagerError {
                        password,
                        password_confirm,
                    });
                }

                Ok(EditPasswordManagerValidated {
                    password: password.expect("Password is not empty"),
                    password_confirm: password_confirm.expect("Password Confirm is not empty"),
                })
            }
            .await,
        )
    }

    pub async fn as_form_html(
        &self,
        context_html_builder: &ContextHtmlBuilder,
        errors: Option<EditPasswordManagerMessage>,
        token: Option<Markup>,
        username: Option<String>,
    ) -> Markup {
        let errors = errors.unwrap_or_default();
        let token = token.unwrap_or_default();
        let user_form_locale = UserFormLocale::new(&context_html_builder.locale);
        let username = username.unwrap_or_default();
        context_html_builder
            .attach_title(&user_form_locale.title_edit_password)
            .attach_content(html! {
                h1 .mt-3 { (user_form_locale.title_edit_password) }
                h2 { (username) }
                form hx-boost="true" hx-target="#main-content" .form method="post" {
                    (token)
                    div .form-group {
                        label .label for="password" { (user_form_locale.password) }
                        input .form-item .w-full type="password" name="password" #password
                        placeholder=(user_form_locale.password_placeholder) {}
                        (errors.password.into_error_html())
                    }
                    div .form-group {
                        label .label for="password-confirm" { (user_form_locale.password_confirm) }
                        input .form-item .w-full type="password" name="password_confirm" #password-confirm
                        placeholder=(user_form_locale.password_confirm_placeholder) {}
                        (errors.password_confirm.into_error_html())
                    }
                    div .form-group {
                        input .btn .btn-sky-blue type="submit" value=(user_form_locale.submit_password) {}
                    }
                }
            })
            .build()
    }
}

pub struct EditPasswordManagerValidated {
    pub password: Password,
    #[allow(dead_code)]
    pub password_confirm: Password,
}

#[cfg(test)]
impl EditPasswordManagerValidated {
    pub fn new_test_data() -> Self {
        Self {
            password: Password::parse(Some("aVHTsh_SEGW5[g_c`/uh>~0!YI0'~fJw"))
                .expect("test password"),
            password_confirm: Password::parse(Some("aVHTsh_SEGW5[g_c`/uh>~0!YI0'~fJw"))
                .expect("test password confirm"),
        }
    }
}

pub struct EditPasswordManagerError {
    pub password: Result<Password, PasswordError>,
    pub password_confirm: Result<Password, PasswordError>,
}

impl EditPasswordManagerError {
    pub fn as_message(&self, locale: &Locale) -> EditPasswordManagerMessage {
        EditPasswordManagerMessage {
            password: self.password.as_translated_message(locale),
            password_confirm: self.password_confirm.as_translated_message(locale),
        }
    }
}

pub struct EditPasswordManagerResult(
    pub Result<EditPasswordManagerValidated, EditPasswordManagerError>,
);

#[derive(Debug, Clone, Serialize, Default)]
pub struct EditPasswordManagerMessage {
    pub password: Arc<[String]>,
    pub password_confirm: Arc<[String]>,
}
