use poem::i18n::{I18NArgs, Locale};
use shared::locale::LocaleExt;

pub struct UserLocale {
    pub user_list_title: String,
    pub user_list_head_id: String,
    pub user_list_head_username: String,
    pub user_list_head_role: String,
    pub user_list_action_edit: String,
    pub user_list_action_password: String,
    pub user_list_action_sign_out: String,
    pub user_list_action_add_user: String,
}

impl UserLocale {
    pub fn new(l: &Locale) -> Self {
        Self {
            user_list_title: l.text_with_default("user-route-list-user-title", "List of Users"),
            user_list_head_id: l.text_with_default("user-route-list-head-id", "Id"),
            user_list_head_username: l
                .text_with_default("user-route-list-head-username", "Username"),
            user_list_head_role: l.text_with_default("user-route-list-head-role", "Role"),
            user_list_action_edit: l.text_with_default("user-route-list-action-edit", "Edit User"),
            user_list_action_password: l
                .text_with_default("user-route-list-action-password", "Edit Password"),
            user_list_action_sign_out: l
                .text_with_default("user-route-list-action-sign-out", "Sign Out User"),
            user_list_action_add_user: l
                .text_with_default("user-route-list-action-add-user", "Add Users"),
        }
    }
}

pub fn user_logout_confirm_message(l: &Locale, username: &str) -> String {
    l.text_with_default_args(
        "user-route-logout-confirm-message",
        format!("Are you sure you want to log out '{username}'?").as_str(),
        I18NArgs::from((("username", username),)),
    )
}
