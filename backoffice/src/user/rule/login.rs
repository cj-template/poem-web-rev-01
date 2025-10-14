use cjtoolkit_structured_validator::types::password::{Password, PasswordError, PasswordRules};
use cjtoolkit_structured_validator::types::username::{Username, UsernameError, UsernameRules};

#[inline]
fn username_rules_for_login() -> UsernameRules {
    UsernameRules {
        is_mandatory: true,
        min_length: None,
        max_length: None,
    }
}

#[inline]
fn password_rules_for_login() -> PasswordRules {
    PasswordRules {
        is_mandatory: true,
        must_have_uppercase: false,
        must_have_lowercase: false,
        must_have_special_chars: false,
        must_have_digit: false,
        min_length: None,
        max_length: Some(64),
    }
}

pub trait UsernameRulesForLoginExt {
    fn parse_user_login(s: Option<&str>) -> Result<Username, UsernameError>;
}

impl UsernameRulesForLoginExt for Username {
    fn parse_user_login(s: Option<&str>) -> Result<Username, UsernameError> {
        Self::parse_custom(s, username_rules_for_login())
    }
}

pub trait PasswordRulesForLoginExt {
    fn parse_user_login(s: Option<&str>) -> Result<Password, PasswordError>;
}

impl PasswordRulesForLoginExt for Password {
    fn parse_user_login(s: Option<&str>) -> Result<Password, PasswordError> {
        Self::parse_custom(s, password_rules_for_login())
    }
}
