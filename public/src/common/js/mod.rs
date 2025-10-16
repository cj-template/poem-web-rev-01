use crate::common::embed::AssetHidden;
use maud::{Markup, PreEscaped, html};
use shared::embed::EmbedAsString;

#[inline]
fn js_debug_prod(debug: &str, prod: &str) -> String {
    if cfg!(debug_assertions) {
        AssetHidden::get(debug).as_string()
    } else {
        AssetHidden::get(prod).as_string()
    }
}

pub fn js_boot() -> String {
    js_debug_prod("js/boot.js", "js/boot.min.js")
}

pub fn js_vec_wrap(vec: Vec<String>) -> Markup {
    html! {
        @for value in vec {
            script type="module" { (PreEscaped(value)) }
        }
    }
}
