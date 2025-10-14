use backoffice::export::{MainError, init_log};
use error_stack::Report;
use error_stack::fmt::ColorMode;
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<(), Report<MainError>> {
    init_log();
    Report::set_color_mode(ColorMode::None);

    let backoffice_handle = tokio::spawn(backoffice::boot());
    let public_handle = tokio::spawn(public::boot());
    match tokio::try_join!(flatten(backoffice_handle), flatten(public_handle)) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

async fn flatten(
    handle: JoinHandle<Result<(), Report<MainError>>>,
) -> Result<(), Report<MainError>> {
    match handle.await {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(err),
        Err(err) => Err(Report::new(err).change_context(MainError::ThreadError)),
    }
}
