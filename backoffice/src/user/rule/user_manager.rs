use cjtoolkit_structured_validator::common::locale::{LocaleData, LocaleMessage, LocaleValue};
use cjtoolkit_structured_validator::common::validation_check::ValidationCheck;
use cjtoolkit_structured_validator::common::validation_collector::ValidateErrorCollector;
use cjtoolkit_structured_validator::types::password::{Password, PasswordError};
use cjtoolkit_structured_validator::types::username::{
    IsUsernameTakenAsync, Username, UsernameError,
};
use paspio::entropy;
use std::sync::Arc;

struct UsernameReservedLocale;

impl LocaleMessage for UsernameReservedLocale {
    fn get_locale_data(&self) -> Arc<LocaleData> {
        LocaleData::new("validate-username-reserved")
    }
}

fn check_username_is_reserved(username: &str) -> Result<(), UsernameError> {
    if username == "visitor" {
        let mut messages = ValidateErrorCollector::new();
        messages.push((
            "Username is reserved".to_string(),
            Box::new(UsernameReservedLocale),
        ));
        UsernameError::validate_check(messages)?;
    }
    Ok(())
}

pub trait UsernameUserManagerRulesExt {
    fn parse_user_add<T: IsUsernameTakenAsync>(
        username: Option<&str>,
        service: &T,
        current_user_name: Option<&str>,
    ) -> impl Future<Output = Result<Username, UsernameError>>;
}

impl UsernameUserManagerRulesExt for Username {
    async fn parse_user_add<T: IsUsernameTakenAsync>(
        username: Option<&str>,
        service: &T,
        current_user_name: Option<&str>,
    ) -> Result<Username, UsernameError> {
        let mut username = Username::parse(username);
        if let Ok(username_ref) = username.as_ref() {
            if let Some(current_user_name) = current_user_name {
                if current_user_name == username_ref.as_str() {
                    return username;
                }
            }
            check_username_is_reserved(username_ref.as_str())?;
            username = username_ref.check_username_taken_async(service).await;
        }
        username
    }
}

pub type PasswordTuple = (
    Result<Password, PasswordError>,
    Result<Password, PasswordError>,
);

pub trait PasswordUserManagerRulesExt {
    fn parse_password_add(s: Option<&str>, password_confirm: &str) -> PasswordTuple;
}

impl PasswordUserManagerRulesExt for Password {
    fn parse_password_add(password: Option<&str>, password_confirm: &str) -> PasswordTuple {
        let password = Password::parse(password);
        if let Ok(password_ref) = password.as_ref() {
            return match PasswordStatus::check(password_ref, password_confirm) {
                PasswordStatus::ConfirmError(err) => (password, Err(err)),
                PasswordStatus::ConfirmOk(ok) => (password, Ok(ok)),
                PasswordStatus::EntropyError(err) => (Err(err), Err(PasswordError::default())),
            };
        }
        (password, Err(PasswordError::default()))
    }
}

pub struct PasswordEntropyLocale(pub f64);

impl LocaleMessage for PasswordEntropyLocale {
    fn get_locale_data(&self) -> Arc<LocaleData> {
        LocaleData::new_with_vec(
            "validate-password-entropy",
            vec![("min".to_string(), LocaleValue::from(self.0))],
        )
    }
}

enum PasswordStatus {
    ConfirmError(PasswordError),
    ConfirmOk(Password),
    EntropyError(PasswordError),
}

impl PasswordStatus {
    fn check(password: &Password, password_confirm: &str) -> Self {
        if let Err(err) = Self::check_password_entropy(password) {
            return Self::EntropyError(err);
        }
        match password.parse_confirm(password_confirm) {
            Ok(ok) => Self::ConfirmOk(ok),
            Err(err) => Self::ConfirmError(err),
        }
    }

    const PASSWORD_ENTROPY_MIN: f64 = 60.0;

    fn check_password_entropy(password: &Password) -> Result<(), PasswordError> {
        let mut messages = ValidateErrorCollector::new();
        if entropy(password.as_str()) < Self::PASSWORD_ENTROPY_MIN {
            messages.push((
                format!(
                    "Password entropy score must be over {}",
                    Self::PASSWORD_ENTROPY_MIN
                ),
                Box::new(PasswordEntropyLocale(Self::PASSWORD_ENTROPY_MIN)),
            ));
        }
        PasswordError::validate_check(messages)?;
        Ok(())
    }
}
