use poem::i18n::{I18NArgs, Locale};
use shared::locale::LocaleExt;

pub struct ShortyRouteLocale {
    pub title: String,
    pub head_id: String,
    pub head_path: String,
    pub head_redirect_url: String,
    pub head_created_at: String,
    pub head_created_by: String,
    pub head_action: String,
    pub action_edit: String,
    pub action_delete: String,
    pub action_add: String,
}

impl ShortyRouteLocale {
    pub fn new(l: &Locale) -> Self {
        Self {
            title: l.text_with_default("shorty-route-title", "Shorty"),
            head_id: l.text_with_default("shorty-route-head-id", "ID"),
            head_path: l.text_with_default("shorty-route-head-path", "Path"),
            head_redirect_url: l
                .text_with_default("shorty-route-head-redirect-url", "Redirect URL"),
            head_created_at: l.text_with_default("shorty-route-head-created-at", "Created At"),
            head_created_by: l.text_with_default("shorty-route-head-created-by", "Created By"),
            head_action: l.text_with_default("shorty-route-head-action", "Action"),
            action_edit: l.text_with_default("shorty-route-action-edit", "Edit Url"),
            action_delete: l.text_with_default("shorty-route-action-delete", "Delete Url"),
            action_add: l.text_with_default("shorty-route-action-add", "Add Url"),
        }
    }
}

pub fn short_route_confirm_message(l: &Locale, id: i64) -> String {
    l.text_with_default_args(
        "shorty-route-confirm-message",
        format!("Are you sure you want to delete '{id}'?").as_str(),
        I18NArgs::from((("id", id),)),
    )
}
