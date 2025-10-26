use crate::context::{Context, ContextError, FromContext};
use error_stack::Report;
use maud::{html, Markup};
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
        match self {
            Self::Success { msg } => {
                html! {
                   div .flash-message .flash-message-success hx-get="/noop" hx-swap="outerHTML swap:5s"
                        hx-target="this" hx-trigger="revealed delay:3s" {
                       (msg)
                   }
                }
            }
            Self::Error { msg } => {
                html! {
                   div .flash-message .flash-message-error hx-get="/noop" hx-swap="outerHTML swap:5s"
                        hx-target="this" hx-trigger="revealed delay:3s" {
                       (msg)
                   }
                }
            }
            Self::Warning { msg } => {
                html! {
                   div .flash-message .flash-message-warning hx-get="/noop" hx-swap="outerHTML swap:5s"
                        hx-target="this" hx-trigger="revealed delay:3s" {
                       (msg)
                   }
                }
            }
        }
    }
}

pub trait FlashMessageHtml {
    fn flash_message_html(&self) -> Markup;
}

impl FlashMessageHtml for Option<Flash> {
    fn flash_message_html(&self) -> Markup {
        match self {
            None => {
                html! {}
            }
            Some(flash) => flash.as_html(),
        }
    }
}

pub trait FlashMessage {
    fn flash(&self, flash: Flash);

    fn get_flash(&self) -> Option<Flash>;
}

impl FlashMessage for Session {
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
