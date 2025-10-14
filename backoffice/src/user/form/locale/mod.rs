use poem::i18n::Locale;
use shared::locale::LocaleExt;

pub struct UserFormLocale {
    pub title_add: String,
    pub title_edit: String,
    pub title_edit_password: String,
    pub username: String,
    pub username_placeholder: String,
    pub password: String,
    pub password_placeholder: String,
    pub password_confirm: String,
    pub password_confirm_placeholder: String,
    pub role: String,
    pub submit_add: String,
    pub submit_edit: String,
    pub submit_password: String,
}

impl UserFormLocale {
    pub fn new(locale: &Locale) -> Self {
        Self {
            title_add: locale.text_with_default("user-form-title-add", "Add User"),
            title_edit: locale.text_with_default("user-form-title-edit", "Edit User"),
            title_edit_password: locale
                .text_with_default("user-form-title-edit-password", "Edit User Password"),
            username: locale.text_with_default("user-form-username", "Username:"),
            username_placeholder: locale
                .text_with_default("user-form-username-placeholder", "Username"),
            password: locale.text_with_default("user-form-password", "Password:"),
            password_placeholder: locale
                .text_with_default("user-form-password-placeholder", "Password"),
            password_confirm: locale
                .text_with_default("user-form-password-confirm", "Password Confirm:"),
            password_confirm_placeholder: locale
                .text_with_default("user-form-password-confirm-placeholder", "Password Confirm"),
            role: locale.text_with_default("user-form-role", "Role:"),
            submit_add: locale.text_with_default("user-form-submit-add", "Add"),
            submit_edit: locale.text_with_default("user-form-submit-edit", "Edit"),
            submit_password: locale.text_with_default("user-form-submit-password", "Submit"),
        }
    }
}
