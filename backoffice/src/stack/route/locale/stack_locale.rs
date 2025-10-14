use poem::i18n::{I18NArgs, Locale};
use shared::locale::LocaleExt;

pub struct StackLocale {
    pub title: String,
    pub head_id: String,
    pub head_name: String,
    pub head_summary: String,
    pub head_reported: String,
    pub head_action: String,
    pub action_details: String,
    pub action_clear: String,
}

impl StackLocale {
    pub fn new(l: &Locale) -> Self {
        Self {
            title: l.text_with_default("stack-list-error-stack-title", "List Error Stack"),
            head_id: l.text_with_default("stack-list-error-stack-head-id", "ID"),
            head_name: l.text_with_default("stack-list-error-stack-head-name", "Name"),
            head_summary: l.text_with_default("stack-list-error-stack-head-summary", "Summary"),
            head_reported: l
                .text_with_default("stack-list-error-stack-head-reported", "Reported At"),
            head_action: l.text_with_default("stack-list-error-stack-head-action", "Action"),
            action_details: l.text_with_default(
                "stack-list-error-stack-action-details",
                "View Error Details",
            ),
            action_clear: l.text_with_default(
                "stack-list-error-stack-action-clear",
                "Clear Older than 30 days",
            ),
        }
    }
}

pub struct StackFetchLocale {
    pub title: String,
    pub head_reported: String,
    pub head_summary: String,
    pub head_stack: String,
}

impl StackFetchLocale {
    pub fn new(l: &Locale, name: &str) -> Self {
        Self {
            title: l.text_with_default_args(
                "stack-list-error-stack-fetch-title",
                format!("Error Stack: {name}").as_str(),
                I18NArgs::from((("name", name),)),
            ),
            head_reported: l
                .text_with_default("stack-list-error-stack-fetch-head-reported", "Reported At"),
            head_summary: l
                .text_with_default("stack-list-error-stack-fetch-head-summary", "Summary"),
            head_stack: l.text_with_default("stack-list-error-stack-fetch-head-stack", "Stack"),
        }
    }
}

pub fn stack_clear_confirm_message(l: &Locale) -> String {
    l.text_with_default(
        "stack-route-logout-confirm-message",
        "Are you sure you want to clear all error stacks older than 30 days?",
    )
}
