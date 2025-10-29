use crate::user::model::user_model::UserIdContext;
use crate::user::service::user_check_service::UserCheckService;
use error_stack::Report;
use shared::utils::context::{Context, ContextError, FromContext};
use shared::utils::request_cache::RequestCacheExt;
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
        let req = ctx.req_result()?;
        req.get_or_init_cache(|| async {
            let user_service: UserCheckService = ctx.inject().await?;
            let user_id_context = user_service.get_user_context();
            Ok(UserPointer(Arc::new(user_id_context)))
        })
        .await
    }
}
