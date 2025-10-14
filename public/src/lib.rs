pub(crate) mod shorty;

use error_stack::{Report, ResultExt};
use poem::middleware::CatchPanic;
use poem::{EndpointExt, IntoResponse, Server};
use shared::config::Config;
use shared::error::boot_error::MainError;
use shared::log::log_poem_error;
use shorty::route::shorty::shorty_route;

pub async fn boot() -> Result<(), Report<MainError>> {
    let config = Config::fetch()
        .await
        .change_context(MainError::ConfigError)?;

    let route = shorty_route();

    let route = route
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
