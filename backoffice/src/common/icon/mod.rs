use crate::common::embed::AssetHidden;
use maud::{Markup, PreEscaped};
use shared::embed::EmbedAsString;

#[inline]
fn get_icon(name: &str) -> Markup {
    PreEscaped(AssetHidden::get(name).as_string())
}

pub fn plus_icon() -> Markup {
    get_icon("icon/plus.svg")
}

pub fn pencil_square_icon() -> Markup {
    get_icon("icon/pencil_square.svg")
}

pub fn key_icon() -> Markup {
    get_icon("icon/key.svg")
}

pub fn flag_icon() -> Markup {
    get_icon("icon/flag.svg")
}

#[allow(dead_code)]
pub fn trash_icon() -> Markup {
    get_icon("icon/trash.svg")
}

pub fn no_symbol_icon() -> Markup {
    get_icon("icon/no_symbol.svg")
}

pub fn document_magnifying_glass_icon() -> Markup {
    get_icon("icon/document_magnifying_glass.svg")
}

pub fn user_minus_icon() -> Markup {
    get_icon("icon/user_minus.svg")
}

pub fn users_icon() -> Markup {
    get_icon("icon/users.svg")
}

pub fn home_icon() -> Markup {
    get_icon("icon/home.svg")
}

pub fn exclamation_circle_icon() -> Markup {
    get_icon("icon/exclamation_circle.svg")
}
