pub(crate) mod common;
pub(crate) mod home;
pub(crate) mod shorty;
pub(crate) mod stack;
pub(crate) mod user;

use crate::common::cache::init_request_cache;
use crate::common::embed::{AssetFilesEndPoint, EMBED_PATH};
use crate::common::locale::build_locale_resources;
use crate::home::home_route;
use crate::shorty::route::shorty::{SHORTY_ROUTE, shorty_route};
use crate::stack::route::stack::{STACK_ROUTE, stack_route};
use crate::user::role::user_role_check::must_be_root;
use crate::user::role::visitor_only::visitor_redirect;
use crate::user::route::login::login_route;
use crate::user::route::user::{USER_ROUTE, user_route};
use error_stack::{Report, ResultExt};
use poem::listener::TcpListener;
use poem::middleware::{CatchPanic, CookieJarManager, Csrf};
use poem::session::{CookieConfig, CookieSession};
use poem::{EndpointExt, IntoResponse, Server};
use shared::config::Config;
use shared::csrf::{CSRF_PATH, route_csrf};
use shared::embed::enforce_min_js_on_prod;
use shared::error::boot_error::MainError;
use shared::htmx::htmx_request_around;
use shared::log::log_poem_error;
use user::route::login::LOGIN_ROUTE;

pub mod export {
    pub use shared::error::boot_error::MainError;
    pub use shared::log::init_log;
}

pub async fn boot() -> Result<(), Report<MainError>> {
    let config = Config::fetch()
        .await
        .change_context(MainError::ConfigError)?;

    let route = home_route();

    let route = route
        .nest(LOGIN_ROUTE, login_route())
        .nest(USER_ROUTE, visitor_redirect(user_route()))
        .nest(SHORTY_ROUTE, visitor_redirect(shorty_route()))
        .nest(CSRF_PATH, route_csrf())
        .nest(STACK_ROUTE, visitor_redirect(must_be_root(stack_route())))
        .nest(
            EMBED_PATH,
            enforce_min_js_on_prod(AssetFilesEndPoint::new()),
        );

    let route = route
        .around(htmx_request_around)
        .around(init_request_cache)
        .data(build_locale_resources().change_context(MainError::LocaleError)?)
        .with(CookieJarManager::new())
        .with(CookieSession::new(CookieConfig::new()))
        .with(Csrf::new())
        .catch_all_error(catch_all_error)
        .with(CatchPanic::new());

    match config.upgrade() {
        Some(config) => {
            println!(
                "Backoffice Listening on http://{}",
                config.poem_backoffice.parse_address()
            );
            Server::new(TcpListener::bind(&config.poem_backoffice.parse_address()))
                .run(route)
                .await
                .change_context(MainError::IoError)
        }
        None => Err(Report::new(MainError::ConfigError)),
    }
}

async fn catch_all_error(err: poem::Error) -> impl IntoResponse {
    log_poem_error(&err).await;
    err.into_response()
}
