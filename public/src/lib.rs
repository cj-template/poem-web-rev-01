pub(crate) mod common;
pub(crate) mod home;

use crate::common::embed::{AssetFilesEndPoint, EMBED_PATH};
use crate::common::locale::build_locale_resources;
use crate::home::route::home_route;
use error_stack::{Report, ResultExt};
use poem::middleware::CatchPanic;
use poem::{EndpointExt, IntoResponse, Server};
use shared::utils::config::Config;
use shared::utils::embed::enforce_min_js_on_prod;
use shared::utils::error::boot_error::MainError;
use shared::utils::log::log_poem_error;
use shared::utils::request_cache::init_request_cache;

pub async fn boot() -> Result<(), Report<MainError>> {
    let config = Config::fetch()
        .await
        .change_context(MainError::ConfigError)?;

    let route = home_route();

    let route = route.nest(
        EMBED_PATH,
        enforce_min_js_on_prod(AssetFilesEndPoint::new()),
    );

    let route = route
        .around(init_request_cache)
        .data(build_locale_resources().change_context(MainError::LocaleError)?)
        .catch_all_error(catch_all_error)
        .with(CatchPanic::new());

    match config.upgrade() {
        Some(config) => {
            println!(
                "Public Listening on http://{}",
                config.poem_public.parse_address()
            );
            Server::new(poem::listener::TcpListener::bind(
                &config.poem_public.parse_address(),
            ))
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
