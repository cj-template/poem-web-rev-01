use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::icon::{document_magnifying_glass_icon, no_symbol_icon};
use crate::stack::route::locale::stack_locale::{
    StackFetchLocale, StackLocale, stack_clear_confirm_message,
};
use crate::stack::service::stack_service::StackService;
use maud::{Markup, html};
use poem::session::Session;
use poem::web::{Path, Redirect};
use poem::{Response, Route, get, handler};
use shared::context::Dep;
use shared::error::FromErrorStack;
use shared::flash::{Flash, FlashMessage};
use shared::htmx::HtmxHeader;

pub const STACK_ROUTE: &str = "/stack";

#[handler]
fn list_error_stack(
    Dep(stack_service): Dep<StackService>,
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
) -> Markup {
    let error_stack_list = stack_service.list_error_stack();
    let open_icon = document_magnifying_glass_icon();
    let clear_icon = no_symbol_icon();

    let lc = StackLocale::new(&context_html_builder.locale);
    let title = lc.title.as_str();

    context_html_builder
        .attach_title(title)
        .set_current_tag("id-tag-stack")
        .attach_content(html! {
            h1 { (title) }
            table .table-full {
                thead {
                    th { (lc.head_id) }
                    th { (lc.head_name) }
                    th { (lc.head_summary) }
                    th { (lc.head_reported) }
                    th .action { (lc.head_action) }
                }
                tbody {
                    @for error_stack in error_stack_list.iter() {
                        tr {
                            td { (error_stack.id) }
                            td { (error_stack.error_name) }
                            td { (error_stack.error_summary) }
                            td .js-date-local { (error_stack.reported_at.to_rfc3339()) }
                            td .action {
                                a .icon href=(format!("{}/view/{}", STACK_ROUTE, error_stack.id))
                                    title=(lc.action_details) hx-get=(format!("{}/view/{}", STACK_ROUTE, error_stack.id))
                                    hx-push-url="true" hx-target="#main-content" { (open_icon) }
                            }
                        }
                    }
                }
            }
            div .text-right .mt-3 {
                a .inline-block hx-confirm=(stack_clear_confirm_message(&context_html_builder.locale)) href=(format!("{}/clear", STACK_ROUTE))
                title=(lc.action_clear) hx-delete=(format!("{}/clear", STACK_ROUTE)) { (clear_icon) }
            }
        })
        .build()
}

#[handler]
fn fetch_error_stack_detail(
    Dep(stack_service): Dep<StackService>,
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Path(view_id): Path<i64>,
) -> poem::Result<Markup> {
    let item = stack_service
        .fetch_error_stack(view_id)
        .map_err(poem::Error::from_error_stack)?;

    let lc = StackFetchLocale::new(&context_html_builder.locale, item.error_name.as_str());
    let title = lc.title.as_str();

    Ok(context_html_builder
        .attach_title(title)
        .attach_content(html! {
            h1 { (title) }
            h2 { (lc.head_reported) }
            pre .pre .js-date-local { (item.reported_at.to_rfc3339()) }
            h2 { (lc.head_summary) }
            pre .pre { (item.error_summary) }
            h2 { (lc.head_stack) }
            pre .pre { (item.error_stack) }
        })
        .build())
}

#[handler]
fn clear(
    Dep(stack_service): Dep<StackService>,
    session: &Session,
    htmx_header: HtmxHeader,
) -> poem::Result<Response> {
    stack_service
        .clear()
        .map_err(poem::Error::from_error_stack)?;

    session.flash(Flash::Success {
        msg: "Successfully clear records older than 30 days".to_string(),
    });
    Ok(htmx_header.do_location(
        Redirect::see_other(STACK_ROUTE.to_owned() + "/"),
        "#main-content",
    ))
}

pub fn stack_route() -> Route {
    Route::new()
        .at("/", get(list_error_stack))
        .at("/view/:view_id", get(fetch_error_stack_detail))
        .at("/clear", get(clear).delete(clear))
}
