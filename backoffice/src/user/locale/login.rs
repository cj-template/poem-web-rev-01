use poem::i18n::Locale;
use shared::locale::LocaleExt;

pub struct LoginLocale {
    pub title: String,
    pub username: String,
    pub password: String,
    pub confirm_button: String,
}

impl LoginLocale {
    pub fn new(locale: &Locale) -> Self {
        Self {
            title: locale.text_with_default("login-title", "User Login"),
            username: locale.text_with_default("login-username", "Username"),
            password: locale.text_with_default("login-password", "Password"),
            confirm_button: locale.text_with_default("login-confirm-button", "Login"),
        }
    }
}

pub struct LoginPostLocale {
    pub flash_success: String,
    pub flash_failed: String,
}

impl LoginPostLocale {
    pub fn new(locale: &Locale) -> Self {
        Self {
            flash_success: locale.text_with_default("login-post-flash-success", "Login success"),
            flash_failed: locale.text_with_default("login-post-flash-failed", "Login failed"),
        }
    }
}

pub struct LogoutLocale {
    pub flash_success: String,
}

impl LogoutLocale {
    pub fn new(locale: &Locale) -> Self {
        Self {
            flash_success: locale.text_with_default("login-logout-post-success", "Logout success"),
        }
    }
}
