use crate::user::pointer::user_pointer::UserPointer;
use poem::Endpoint;
use shared::context::Context;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default)]
pub struct BackofficeRequestCache {
    pub user_pointer: Option<UserPointer>,
}

pub async fn init_request_cache<EP: Endpoint>(
    next: EP,
    mut req: poem::Request,
) -> poem::Result<EP::Output> {
    req.set_data(Arc::new(Mutex::new(BackofficeRequestCache::default())));
    next.call(req).await
}

pub trait RequestCacheExt {
    fn request_cache(&self) -> Arc<Mutex<BackofficeRequestCache>>;
}

impl RequestCacheExt for poem::Request {
    fn request_cache(&self) -> Arc<Mutex<BackofficeRequestCache>> {
        Arc::clone(
            self.data::<Arc<Mutex<BackofficeRequestCache>>>()
                .expect("Request Cache"),
        )
    }
}

impl RequestCacheExt for Context<'_> {
    fn request_cache(&self) -> Arc<Mutex<BackofficeRequestCache>> {
        self.req_result().expect("Request").request_cache()
    }
}
