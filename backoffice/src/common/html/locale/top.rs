use poem::i18n::{I18NArgs, Locale};
use shared::locale::LocaleExt;

pub struct TopBuildLocale {
    pub hello: String,
    pub hello_logout: String,
    pub visitor: String,
}

impl TopBuildLocale {
    pub fn new(locale: &Locale, username: &str) -> Self {
        Self {
            hello: locale.text_with_default_args(
                "top-hello",
                format!("Hello, {}", username).as_str(),
                I18NArgs::from((("username", username),)),
            ),
            hello_logout: locale.text_with_default("top-hello-logout", "Click here to logout"),
            visitor: locale
                .text_with_default("top-visitor", "You're a visitor, click here to login"),
        }
    }
}
