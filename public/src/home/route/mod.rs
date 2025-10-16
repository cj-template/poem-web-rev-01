use crate::common::embed::AssetFileEndPoint;
use crate::common::html::HtmlBuilder;
use maud::{Markup, html};
use poem::i18n::Locale;
use poem::{get, handler};
use shared::locale::LocaleExt;

#[handler]
async fn home(locale: Locale) -> Markup {
    let title = locale.text_with_default("home-title", "Welcome");
    let hello = locale.text_with_default("home-hello", "Hello");
    let note = locale.text_with_default("home-note", "Welcome to the beginning of the journey");

    HtmlBuilder::new(
        title,
        html! {
            div .home-content {
                h1 .hello { (hello) }
                p .note { (note) }
            }
        },
    )
    .attach_head(html! {})
    .attach_footer(html! {})
    .build()
}

pub fn home_route() -> poem::Route {
    poem::Route::new().at("/", get(home)).at(
        "/favicon.ico",
        AssetFileEndPoint::new("favicon/favicon.ico"),
    )
}
