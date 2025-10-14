use crate::common::embed::AssetFileEndPoint;
use crate::common::html::context_html::ContextHtmlBuilder;
use crate::user::role::visitor_only::visitor_redirect;
use maud::{Markup, html};
use poem::{Route, get, handler};
use shared::context::Dep;

#[handler]
pub async fn home_page(Dep(context_html_builder): Dep<ContextHtmlBuilder>) -> Markup {
    let title = context_html_builder
        .locale
        .text("home-hello")
        .unwrap_or("Welcome".to_string());
    let paragraph = context_html_builder
        .locale
        .text("home-paragraph")
        .unwrap_or("Check out the navigation above".to_string());
    context_html_builder
        .attach_title(&title)
        .set_current_tag("id-tag-home")
        .attach_content(html! {
            h1 .mt-3 { (title) }
            p { (paragraph) }
        })
        .build()
}

pub fn home_route() -> Route {
    Route::new().at("/", visitor_redirect(get(home_page))).at(
        "/favicon.ico",
        AssetFileEndPoint::new("favicon/favicon.ico"),
    )
}
