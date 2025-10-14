use poem::i18n::Locale;
use shared::locale::LocaleExt;

pub struct ShortyFormLocale {
    pub title_edit: String,
    pub title_add: String,
    pub url_path: String,
    pub url_path_placeholder: String,
    pub url_redirect: String,
    pub url_redirect_placeholder: String,
    pub submit_button: String,
}

impl ShortyFormLocale {
    pub fn new(l: &Locale) -> Self {
        Self {
            title_edit: l.text_with_default("shorty-form-title-edit", "Edit Url"),
            title_add: l.text_with_default("shorty-form-title-add", "Add Url"),
            url_path: l.text_with_default("shorty-form-url-path", "Path:"),
            url_path_placeholder: l.text_with_default("shorty-form-url-path-placeholder", "Path"),
            url_redirect: l.text_with_default("shorty-form-url-redirect", "Redirect To:"),
            url_redirect_placeholder: l
                .text_with_default("shorty-form-url-redirect-placeholder", "Redirect To"),
            submit_button: l.text_with_default("shorty-form-submit-button", "Save"),
        }
    }
}
