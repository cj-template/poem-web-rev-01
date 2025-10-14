use crate::user::rule::login::{PasswordRulesForLoginExt, UsernameRulesForLoginExt};
use cjtoolkit_structured_validator::common::flag_error::FlagCounter;
use cjtoolkit_structured_validator::types::password::{Password, PasswordError};
use cjtoolkit_structured_validator::types::username::{Username, UsernameError};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct UserLoginForm {
    pub username: String,
    pub password: String,
    pub csrf_token: String,
}

impl UserLoginForm {
    pub fn as_validated(&self) -> UserLoginFormResult {
        UserLoginFormResult((|| {
            let mut flag = FlagCounter::new();

            let username = flag.check(Username::parse_user_login(Some(
                self.username.as_str().trim(),
            )));
            let password = flag.check(Password::parse_user_login(Some(
                self.password.as_str().trim(),
            )));

            if flag.is_flagged() {
                return Err(UserLoginFormError { username, password });
            }

            Ok(UserLoginFormValidated {
                username: username.expect("Username is not empty"),
                password: password.expect("Password is not empty"),
            })
        })())
    }
}

pub struct UserLoginFormValidated {
    pub username: Username,
    pub password: Password,
}

#[allow(dead_code)]
pub struct UserLoginFormError {
    pub username: Result<Username, UsernameError>,
    pub password: Result<Password, PasswordError>,
}

pub struct UserLoginFormResult(pub Result<UserLoginFormValidated, UserLoginFormError>);
