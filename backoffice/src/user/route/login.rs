use crate::common::html::context_html::ContextHtmlBuilder;
use crate::user::LOGIN_TOKEN_COOKIE_NAME;
use crate::user::form::login::{UserLoginForm, UserLoginFormResult};
use crate::user::locale::login::{LoginLocale, LoginPostLocale, LogoutLocale};
use crate::user::role::user_role_check::must_be_user;
use crate::user::role::visitor_only::visitor_only;
use crate::user::service::user_login_service::UserLoginService;
use chrono::TimeDelta;
use error_stack::Report;
use maud::{Markup, html};
use poem::error::ResponseError;
use poem::i18n::Locale;
use poem::session::Session;
use poem::web::cookie::{Cookie, CookieJar};
use poem::web::{CsrfToken, CsrfVerifier, Redirect};
use poem::{IntoResponse, Response, Route, get, handler};
use shared::adapter::unified;
use shared::context::Dep;
use shared::cookie_builders::CookieBuilderExt;
use shared::csrf::{CsrfError, CsrfTokenHtml, CsrfVerifierError};
use shared::flash::{Flash, FlashMessage};
use shared::query_string::form::FormQs;

pub const LOGIN_ROUTE: &str = "/user-login";

#[handler]
async fn login(
    Dep(context_html_builder): Dep<ContextHtmlBuilder>,
    csrf_token: &CsrfToken,
) -> Markup {
    let login_locale = LoginLocale::new(&context_html_builder.locale);
    context_html_builder
        .attach_title(&login_locale.title)
        .attach_content(html! {
            h1 .mt-3 { (login_locale.title) }
            form method="post" .form {
                (csrf_token.as_html())
                input .form-item type="text" name="username" placeholder=(login_locale.username) {}
                input .form-item type="password" name="password" placeholder=(login_locale.password) {}
                button .btn .btn-sky-blue .mt-3 type="submit" { (login_locale.confirm_button) }
            }
        })
        .build()
}

enum LoginPostResponse {
    Redirect(Redirect),
    CsrfError(Report<CsrfError>),
}

impl IntoResponse for LoginPostResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Redirect(redirect) => redirect.into_response(),
            Self::CsrfError(csrf) => csrf.current_context().as_response(),
        }
    }
}

#[handler]
async fn login_post(
    Dep(user_login_service): Dep<UserLoginService>,
    FormQs(user_login_form): FormQs<UserLoginForm>,
    session: &Session,
    cookie_jar: &CookieJar,
    csrf_verifier: &CsrfVerifier,
    locale: Locale,
) -> LoginPostResponse {
    unified(async {
        csrf_verifier
            .verify(user_login_form.csrf_token.as_str())
            .map_err(|err| LoginPostResponse::CsrfError(err))?;
        let login_post_locale = LoginPostLocale::new(&locale);
        if let UserLoginFormResult(Ok(user_login_form_validated)) = user_login_form.as_validated() {
            let token = user_login_service.validate_login(
                user_login_form_validated.username.as_str().to_string(),
                user_login_form_validated.password.as_str().to_string(),
            );
            if let Some(token) = token {
                let new_cookie = Cookie::new_with_str(LOGIN_TOKEN_COOKIE_NAME, token)
                    .into_builder()
                    .path("/")
                    .expires_by_delta(TimeDelta::days(30))
                    .secure()
                    .http_only()
                    .build();

                cookie_jar.add(new_cookie);
                session.flash(Flash::Success {
                    msg: login_post_locale.flash_success,
                });
                return Ok(LoginPostResponse::Redirect(Redirect::see_other("/")));
            }
        }

        session.flash(Flash::Error {
            msg: login_post_locale.flash_failed,
        });
        Err(LoginPostResponse::Redirect(Redirect::see_other(
            LOGIN_ROUTE.to_owned() + "/",
        )))
    })
    .await
}

#[handler]
async fn logout(
    Dep(user_login_service): Dep<UserLoginService>,
    session: &Session,
    cookie_jar: &CookieJar,
    locale: Locale,
) -> Redirect {
    user_login_service.logout();
    cookie_jar.remove(LOGIN_TOKEN_COOKIE_NAME);
    let logout_locale = LogoutLocale::new(&locale);
    session.flash(Flash::Success {
        msg: logout_locale.flash_success,
    });
    Redirect::see_other("/")
}

pub fn login_route() -> Route {
    Route::new()
        .at("/", visitor_only(get(login).post(login_post)))
        .at("/logout", must_be_user(get(logout)))
}
