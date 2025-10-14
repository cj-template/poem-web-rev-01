use crate::common::html::context_html::ContextHtmlBuilder;
use crate::common::icon::{flag_icon, key_icon, pencil_square_icon, plus_icon};
use crate::user::form::add_user::AddUserForm;
use crate::user::form::edit_password_manager::EditPasswordManagerForm;
use crate::user::form::edit_user::EditUserForm;
use crate::user::locale::user::{UserLocale, user_logout_confirm_message};
use crate::user::pointer::user_pointer::UserPointer;
use crate::user::repository::user_manager_repository::UserManagerRepository;
use crate::user::role::Role;
use crate::user::role::user_role_check::{must_be_root, must_be_user};
use crate::user::service::user_manager_service::add_user_service::AddUserService;
use crate::user::service::user_manager_service::edit_password_service::EditPasswordService;
use crate::user::service::user_manager_service::edit_service::EditUserService;
use crate::user::service::user_manager_service::list_service::ListUserService;
use maud::{Markup, html};
use poem::http::StatusCode;
use poem::i18n::{I18NArgs, Locale};
use poem::session::Session;
use poem::web::{CsrfToken, CsrfVerifier, Path, Redirect};
use poem::{Error, IntoResponse, Response, Route, get, handler};
use shared::context::Dep;
use shared::csrf::{CsrfTokenHtml, CsrfVerifierError};
use shared::error::{ExtraResultExt, FromErrorStack};
use shared::flash::{Flash, FlashMessage};
use shared::htmx::HtmxHeader;
use shared::locale::LocaleExt;
use shared::query_string::form::FormQs;

pub const USER_ROUTE: &str = "/user";

#[handler]
async fn list_users(
    Dep(list_user_service): Dep<ListUserService>,
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(user_id_context): Dep<UserPointer>,
) -> Markup {
    let list_user = list_user_service.list_users();
    let edit_icon = pencil_square_icon();
    let password_icon = key_icon();
    let flag_icon = flag_icon();

    let user_locale = UserLocale::new(&context_html_builder.locale);

    context_html_builder
        .attach_title(&user_locale.user_list_title)
        .set_current_tag("id-tag-user")
        .attach_content(html! {
            h1 { (&user_locale.user_list_title) }
            table .table-full {
                thead {
                    tr {
                        th { (&user_locale.user_list_head_id) }
                        th { (&user_locale.user_list_head_username) }
                        th { (&user_locale.user_list_head_role) }
                        @if user_id_context.role == Role::Root {
                            th .action { "Action" }
                        }
                    }
                }
                tbody {
                    @for user in list_user.iter() {
                        tr {
                            td { (user.id) }
                            td { (&user.username) }
                            td { (user.role.as_stringed()) }
                            @if user_id_context.role == Role::Root {
                                td .action {
                                    a .icon href=(format!("{}/edit/{}", USER_ROUTE, user.id)) title=(&user_locale.user_list_action_edit)
                                        hx-get=(format!("{}/edit/{}", USER_ROUTE, user.id)) hx-push-url="true" hx-target="#main-content" { (edit_icon) }
                                    " "
                                    a .icon href=(format!("{}/edit-password/{}", USER_ROUTE, user.id)) title=(&user_locale.user_list_action_password)
                                        hx-get=(format!("{}/edit-password/{}", USER_ROUTE, user.id)) hx-push-url="true" hx-target="#main-content" { (password_icon) }
                                    " "
                                    a .icon hx-confirm=(user_logout_confirm_message(&context_html_builder.locale, &user.username))
                                        href=(format!("{}/sign-out/{}", USER_ROUTE, user.id)) title=(&user_locale.user_list_action_sign_out)
                                        hx-get=(format!("{}/sign-out/{}", USER_ROUTE, user.id)) hx-push-url="true" hx-target="#main-content" { (flag_icon) }
                                }
                            }
                        }
                    }
                }
            }
            @if user_id_context.role == Role::Root {
                div .text-right .mt-3 {
                    a .inline-block href=(format!("{}/add-user", USER_ROUTE)) title=(&user_locale.user_list_action_add_user)
                        hx-get=(format!("{}/add-user", USER_ROUTE)) hx-push-url="true" hx-target="#main-content" { (plus_icon()) }
                }
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
async fn edit_user_get(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_user_service): Dep<EditUserService>,
    Path(user_id): Path<i64>,
    csrf_token: &CsrfToken,
) -> poem::Result<Markup> {
    let subject_user = edit_user_service
        .fetch_user(user_id)
        .map_err(Error::from_error_stack)?;

    let mut edit_user = EditUserForm::default();
    edit_user.username = subject_user.username.to_string();
    edit_user.role = subject_user.role;

    Ok(edit_user
        .as_form_html(
            &context_html_builder,
            None,
            Some(csrf_token.as_html()),
            Some(subject_user.username),
        )
        .await)
}

#[handler]
async fn edit_user_post(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_user_service): Dep<EditUserService>,
    Path(user_id): Path<i64>,
    FormQs(edit_user_form): FormQs<EditUserForm>,
    csrf_token: &CsrfToken,
    csrf_verifier: &CsrfVerifier,
    session: &Session,
    htmx_header: HtmxHeader,
) -> poem::Result<Response> {
    let subject_user = edit_user_service
        .fetch_user(user_id)
        .map_err(Error::from_error_stack)?;
    csrf_verifier
        .verify(edit_user_form.csrf_token.as_str())
        .map_err(Error::from_error_stack)?;
    let validated_result = edit_user_form
        .as_validated(&edit_user_service, &subject_user.username)
        .await
        .0;
    let l = &context_html_builder.locale;
    match validated_result {
        Ok(validated) => {
            edit_user_service
                .edit_user_submit(user_id, &validated)
                .log_it()
                .map_err(Error::from_error_stack)?;
            session.flash(Flash::Success {
                msg: l.text_with_default_args(
                    "user-route-flash-edit-success",
                    format!("Successfully edited user id: {}", user_id).as_str(),
                    I18NArgs::from((("user_id", user_id),)),
                ),
            });
            Ok(htmx_header.do_location(
                Redirect::see_other(USER_ROUTE.to_owned() + "/"),
                "#main-content",
            ))
        }
        Err(error) => {
            let errors = error.as_message(&context_html_builder.locale);
            context_html_builder.attach_form_flash_error();
            Ok(PostResponse::Validation(
                edit_user_form
                    .as_form_html(
                        &context_html_builder,
                        Some(errors),
                        Some(csrf_token.as_html()),
                        Some(subject_user.username),
                    )
                    .await,
            )
            .into_response())
        }
    }
}

#[handler]
async fn edit_user_password_get(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_password_service): Dep<EditPasswordService>,
    Path(user_id): Path<i64>,
    csrf_token: &CsrfToken,
) -> poem::Result<Markup> {
    let subject_user = edit_password_service
        .fetch_user(user_id)
        .map_err(Error::from_error_stack)?;

    let edit_password_form = EditPasswordManagerForm::default();

    Ok(edit_password_form
        .as_form_html(
            &context_html_builder,
            None,
            Some(csrf_token.as_html()),
            Some(subject_user.username),
        )
        .await)
}

#[handler]
async fn edit_user_password_post(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(edit_password_service): Dep<EditPasswordService>,
    Path(user_id): Path<i64>,
    FormQs(edit_password_manager_form): FormQs<EditPasswordManagerForm>,
    csrf_token: &CsrfToken,
    csrf_verifier: &CsrfVerifier,
    session: &Session,
    htmx_header: HtmxHeader,
) -> poem::Result<Response> {
    let subject_user = edit_password_service
        .fetch_user(user_id)
        .map_err(Error::from_error_stack)?;
    csrf_verifier
        .verify(edit_password_manager_form.csrf_token.as_str())
        .map_err(Error::from_error_stack)?;
    let validated_result = edit_password_manager_form.as_validated().await.0;
    let l = &context_html_builder.locale;
    match validated_result {
        Ok(validated) => {
            edit_password_service
                .edit_password_submit(user_id, &validated)
                .log_it()
                .map_err(Error::from_error_stack)?;
            session.flash(Flash::Success {
                msg: l.text_with_default_args(
                    "user-route-flash-password-success",
                    format!("Successfully edited password for user id: {}", user_id).as_str(),
                    I18NArgs::from((("user_id", user_id),)),
                ),
            });
            Ok(htmx_header.do_location(
                Redirect::see_other(USER_ROUTE.to_owned() + "/"),
                "#main-content",
            ))
        }
        Err(error) => {
            let errors = error.as_message(&context_html_builder.locale);
            context_html_builder.attach_form_flash_error();
            Ok(PostResponse::Validation(
                edit_password_manager_form
                    .as_form_html(
                        &context_html_builder,
                        Some(errors),
                        Some(csrf_token.as_html()),
                        Some(subject_user.username),
                    )
                    .await,
            )
            .into_response())
        }
    }
}

#[handler]
async fn add_user_password_get(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    csrf_token: &CsrfToken,
) -> poem::Result<Markup> {
    let add_user_form = AddUserForm::default();

    Ok(add_user_form
        .as_form_html(&context_html_builder, None, Some(csrf_token.as_html()))
        .await)
}

#[handler]
async fn add_user_password_post(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    Dep(add_user_service): Dep<AddUserService>,
    FormQs(add_user_form): FormQs<AddUserForm>,
    csrf_token: &CsrfToken,
    csrf_verifier: &CsrfVerifier,
    session: &Session,
    htmx_header: HtmxHeader,
) -> poem::Result<Response> {
    csrf_verifier
        .verify(add_user_form.csrf_token.as_str())
        .map_err(Error::from_error_stack)?;
    let validated_result = add_user_form.as_validated(&add_user_service).await.0;
    let l = &context_html_builder.locale;
    match validated_result {
        Ok(validated) => {
            add_user_service
                .add_user_submit(&validated)
                .log_it()
                .map_err(Error::from_error_stack)?;

            session.flash(Flash::Success {
                msg: l.text_with_default_args(
                    "user-route-flash-add-success",
                    format!("Successfully created user: {}", validated.username.as_str()).as_str(),
                    I18NArgs::from((("username", validated.username.as_str()),)),
                ),
            });
            Ok(htmx_header.do_location(
                Redirect::see_other(USER_ROUTE.to_owned() + "/"),
                "#main-content",
            ))
        }
        Err(error) => {
            let errors = error.as_message(&context_html_builder.locale);
            context_html_builder.attach_form_flash_error();
            Ok(PostResponse::Validation(
                add_user_form
                    .as_form_html(
                        &context_html_builder,
                        Some(errors),
                        Some(csrf_token.as_html()),
                    )
                    .await,
            )
            .into_response())
        }
    }
}

#[handler]
fn sign_out_user(
    Dep(user_manager_repository): Dep<UserManagerRepository>,
    Path(user_id): Path<i64>,
    session: &Session,
    locale: Locale,
    htmx_header: HtmxHeader,
) -> Response {
    let result = user_manager_repository.revoke_all_token_by_id(user_id);
    let l = &locale;
    if result.is_err() {
        session.flash(Flash::Error {
            msg: l.text_with_default_args(
                "user-route-flash-sign-out-error",
                format!("Failed to sign out user id: {}", user_id).as_str(),
                I18NArgs::from((("user_id", user_id),)),
            ),
        });
        return htmx_header.do_location(
            Redirect::see_other(USER_ROUTE.to_owned() + "/"),
            "#main-content",
        );
    }
    session.flash(Flash::Success {
        msg: l.text_with_default_args(
            "user-route-flash-sign-out-success",
            format!("Successfully signed out user id: {}", user_id).as_str(),
            I18NArgs::from((("user_id", user_id),)),
        ),
    });
    htmx_header.do_location(
        Redirect::see_other(USER_ROUTE.to_owned() + "/"),
        "#main-content",
    )
}

pub fn user_route() -> Route {
    Route::new()
        .at("/", get(must_be_user(list_users)))
        .at(
            "/edit/:user_id",
            must_be_root(get(edit_user_get).post(edit_user_post)),
        )
        .at(
            "/edit-password/:user_id",
            must_be_root(get(edit_user_password_get).post(edit_user_password_post)),
        )
        .at(
            "/add-user",
            must_be_root(get(add_user_password_get).post(add_user_password_post)),
        )
        .at("/sign-out/:user_id", must_be_root(get(sign_out_user)))
}
