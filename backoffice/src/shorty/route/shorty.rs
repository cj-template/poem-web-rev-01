use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::icon::{pencil_square_icon, plus_icon, trash_icon};
use crate::shorty::form::add_edit_url_form::AddEditUrlForm;
use crate::shorty::route::locale::shorty::{ShortyRouteLocale, short_route_confirm_message};
use crate::shorty::service::add_url_service::AddUrlService;
use crate::shorty::service::delete_url_service::DeleteUrlService;
use crate::shorty::service::edit_url_service::EditUrlService;
use crate::shorty::service::list_url_service::ListUrlService;
use crate::user::pointer::user_pointer::UserPointer;
use crate::user::role::Role;
use crate::user::role::user_role_check::must_be_user;
use maud::{Markup, html};
use poem::http::StatusCode;
use poem::i18n::Locale;
use poem::session::Session;
use poem::web::{CsrfToken, CsrfVerifier, Path, Redirect};
use poem::{Error, IntoResponse, Response, Route, get, handler};
use shared::context::Dep;
use shared::csrf::{CsrfTokenHtml, CsrfVerifierError};
use shared::error::{ExtraResultExt, FromErrorStack};
use shared::flag::{Flag, flag_add, flag_edit};
use shared::flag::path_edit::PathEdit;
use shared::flash::{Flash, FlashMessage};
use shared::htmx::HtmxHeader;
use shared::locale::LocaleExt;
use shared::query_string::form::FormQs;

pub const SHORTY_ROUTE: &str = "/shorty";

#[handler]
async fn list_urls(
    Dep(list_url_service): Dep<ListUrlService>,
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(user_id_context): Dep<UserPointer>,
) -> Markup {
    let list_urls = list_url_service.list_urls();
    let edit_icon = pencil_square_icon();
    let delete_icon = trash_icon();
    let add_icon = plus_icon();

    let lc = ShortyRouteLocale::new(&context_html_builder.locale);

    context_html_builder
        .attach_title(&lc.title)
        .set_current_tag("id-tag-shorty")
        .attach_content(html! {
            h1 { (lc.title) }
            table .table-full {
                thead {
                    tr {
                        th { (lc.head_id) }
                        th { (lc.head_path) }
                        th { (lc.head_redirect_url) }
                        th { (lc.head_created_at) }
                        th { (lc.head_created_by) }
                        th .action { (lc.head_action) }
                    }
                }
                tbody {
                    @for url in list_urls.iter() {
                        tr {
                            td { (url.id) }
                            td { (url.url_path) }
                            td { (url.url_redirect) }
                            td .js-date-local { (url.created_at.to_rfc3339()) }
                            td { (url.username) }
                            td .action {
                                @if user_id_context.role == Role::Root || user_id_context.id == url.created_by_user_id {
                                    a .icon href=( format!("{}/edit/{}", SHORTY_ROUTE, url.id)) title=(lc.action_edit)
                                        hx-get=( format!("{}/edit/{}", SHORTY_ROUTE, url.id)) hx-target="#main-content" hx-push-url="true" { (edit_icon) }
                                    " "
                                    a .icon hx-confirm=(short_route_confirm_message(&context_html_builder.locale ,url.id))
                                        href=( format!("{}/delete/{}", SHORTY_ROUTE, url.id)) title=(lc.action_delete)
                                        hx-delete=( format!("{}/delete/{}", SHORTY_ROUTE, url.id)) hx-target="#main-content" { (delete_icon) }
                                }
                            }
                        }
                    }
                }
            }
            div .text-right .mt-3 {
                a .inline-block href=( format!("{}/add", SHORTY_ROUTE)) title=(lc.action_add)
                    hx-get=( format!("{}/add", SHORTY_ROUTE)) hx-target="#main-content" hx-push-url="true" { (add_icon) }
            }
        })
        .build()
}

enum PostResponse {
    Validation(Markup),
}

impl IntoResponse for PostResponse {
    fn into_response(self) -> poem::Response {
        match self {
            PostResponse::Validation(validation) => validation
                .with_status(StatusCode::UNPROCESSABLE_ENTITY)
                .into_response(),
        }
    }
}

#[handler]
async fn url_get(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_url_service): Dep<EditUrlService>,
    Dep(user_id_context): Dep<UserPointer>,
    PathEdit(url_id): PathEdit<i64>,
    csrf_token: &CsrfToken,
    flag: Flag,
) -> poem::Result<Markup> {
    let mut url_form = AddEditUrlForm::default();
    if flag.is_edit() {
        let subject_id = edit_url_service
            .fetch_user_id_from_url_id(url_id)
            .map_err(Error::from_error_stack)?;
        if user_id_context.role < Role::Root && user_id_context.id != subject_id.created_by_user_id
        {
            return Err(Error::from_status(StatusCode::FORBIDDEN));
        }
        let subject_url = edit_url_service
            .get_url_redirect(url_id)
            .map_err(Error::from_error_stack)?;
        url_form.url_path = subject_url.url_path;
        url_form.url_redirect = subject_url.url_redirect;
    }

    Ok(url_form
        .as_form_html(
            &context_html_builder,
            None,
            Some(csrf_token.as_html()),
            flag.is_edit(),
        )
        .await)
}

#[handler]
async fn url_post(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_url_service): Dep<EditUrlService>,
    Dep(add_url_service): Dep<AddUrlService>,
    Dep(user_id_context): Dep<UserPointer>,
    PathEdit(url_id): PathEdit<i64>,
    FormQs(edit_url_form): FormQs<AddEditUrlForm>,
    csrf_token: &CsrfToken,
    csrf_verifier: &CsrfVerifier,
    session: &Session,
    htmx_header: HtmxHeader,
    flag: Flag,
) -> poem::Result<Response> {
    if flag.is_edit() {
        let subject_id = edit_url_service
            .fetch_user_id_from_url_id(url_id)
            .map_err(Error::from_error_stack)?;
        if user_id_context.role < Role::Root && user_id_context.id != subject_id.created_by_user_id
        {
            return Err(Error::from_status(StatusCode::FORBIDDEN));
        }
    }
    csrf_verifier
        .verify(edit_url_form.csrf_token.as_str())
        .map_err(Error::from_error_stack)?;
    let validated_result = edit_url_form.as_validated().await.0;
    match validated_result {
        Ok(validated) => {
            let l = &context_html_builder.locale;
            if flag.is_edit() {
                edit_url_service
                    .edit_url_submit(&validated, url_id)
                    .log_it()
                    .map_err(Error::from_error_stack)?;
                session.flash(Flash::Success {
                    msg: l.text_with_default(
                        "shorty-route-flash-success-edit-url",
                        "Successfully edited URL",
                    ),
                });
            } else if flag.is_add() {
                add_url_service
                    .add_url_submit(&validated, user_id_context.id)
                    .log_it()
                    .map_err(Error::from_error_stack)?;
                session.flash(Flash::Success {
                    msg: l.text_with_default(
                        "shorty-route-flash-success-add-url",
                        "Successfully added URL",
                    ),
                });
            }

            Ok(htmx_header.do_location(
                Redirect::see_other(SHORTY_ROUTE.to_owned() + "/"),
                "#main-content",
            ))
        }
        Err(error) => {
            let errors = error.as_message(&context_html_builder.locale);
            context_html_builder.attach_form_flash_error();
            Ok(PostResponse::Validation(
                edit_url_form
                    .as_form_html(
                        &context_html_builder,
                        Some(errors),
                        Some(csrf_token.as_html()),
                        flag.is_edit(),
                    )
                    .await,
            )
            .into_response())
        }
    }
}

#[handler]
async fn delete_url(
    Dep(delete_url_service): Dep<DeleteUrlService>,
    Dep(user_id_context): Dep<UserPointer>,
    Path(url_id): Path<i64>,
    session: &Session,
    l: Locale,
    htmx_header: HtmxHeader,
) -> poem::Result<Response> {
    let subject_id = delete_url_service
        .fetch_user_id_from_url_id(url_id)
        .map_err(Error::from_error_stack)?;
    if user_id_context.role < Role::Root && user_id_context.id != subject_id.created_by_user_id {
        return Err(Error::from_status(StatusCode::FORBIDDEN));
    }
    delete_url_service
        .delete_url(url_id)
        .log_it()
        .map_err(Error::from_error_stack)?;
    session.flash(Flash::Success {
        msg: l.text_with_default(
            "shorty-route-flash-success-deleted-url",
            "Successfully deleted URL",
        ),
    });
    Ok(htmx_header.do_location(
        Redirect::see_other(SHORTY_ROUTE.to_owned() + "/"),
        "#main-content",
    ))
}

pub fn shorty_route() -> Route {
    Route::new()
        .at("/", must_be_user(get(list_urls)))
        .at(
            "/edit/:url_id",
            must_be_user(flag_edit(get(url_get).post(url_post))),
        )
        .at(
            "/delete/:url_id",
            must_be_user(get(delete_url).delete(delete_url)),
        )
        .at("/add", must_be_user(flag_add(get(url_get).post(url_post))))
}
