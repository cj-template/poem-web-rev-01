use crate::utils::context::{Context, ContextError, FromContext};
use error_stack::Report;
use maud::{Markup, html};
use poem::session::Session;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Flash {
    Success { msg: String },
    Error { msg: String },
    Warning { msg: String },
}

impl Flash {
    pub fn as_html(&self) -> Markup {
        let js_date = include_str!("_js/flash_data.js");
        match self {
            Self::Success { msg } => {
                html! {
                   div .flash-message .flash-message-success
                        x-data=(js_date) x-init="start()"
                        x-show="show" "x-transition.duration.400ms" {
                       (msg)
                   }
                }
            }
            Self::Error { msg } => {
                html! {
                   div .flash-message .flash-message-error
                        x-data=(js_date) x-init="start()"
                        x-show="show" "x-transition.duration.400ms" {
                       (msg)
                   }
                }
            }
            Self::Warning { msg } => {
                html! {
                   div .flash-message .flash-message-warning
                        x-data=(js_date) x-init="start()"
                        x-show="show" "x-transition.duration.400ms" {
                       (msg)
                   }
                }
            }
        }
    }
}

pub trait FlashMessageHtmlExt {
    fn flash_message_html(&self) -> Markup;
}

impl FlashMessageHtmlExt for Option<Flash> {
    fn flash_message_html(&self) -> Markup {
        match self {
            None => {
                html! {}
            }
            Some(flash) => flash.as_html(),
        }
    }
}

pub trait FlashMessageExt {
    fn flash(&self, flash: Flash);

    fn get_flash(&self) -> Option<Flash>;
}

impl FlashMessageExt for Session {
    fn flash(&self, flash: Flash) {
        self.set("flash", flash)
    }

    fn get_flash(&self) -> Option<Flash> {
        let flash = self.get("flash");
        self.remove("flash");
        flash
    }
}

impl FromContext for Option<Flash> {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let req = ctx.req_result()?;
        Ok(match req.data::<Session>() {
            None => None,
            Some(session) => session.get_flash(),
        })
    }
}
