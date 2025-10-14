use crate::common::cache::RequestCacheExt;
use crate::user::model::user_model::UserIdContext;
use crate::user::service::user_check_service::UserCheckService;
use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};
use std::ops::Deref;
use std::sync::Arc;

pub struct UserPointer(Arc<UserIdContext>);

impl Clone for UserPointer {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl Deref for UserPointer {
    type Target = UserIdContext;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromContext for UserPointer {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let request_cache = ctx.request_cache();
        let mut request_cache = request_cache.lock().await;
        let user_pointer = match request_cache.user_pointer.as_ref() {
            None => {
                let user_service: UserCheckService = ctx.inject().await?;
                let user_id_context = user_service.get_user_context();
                let user_pointer = UserPointer(Arc::new(user_id_context));
                request_cache.user_pointer = Some(user_pointer.clone());
                user_pointer
            }
            Some(user_pointer) => user_pointer.clone(),
        };
        drop(request_cache);
        Ok(user_pointer)
    }
}
