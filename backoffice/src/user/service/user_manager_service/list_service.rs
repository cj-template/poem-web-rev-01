use crate::user::model::user_manager_model::ListUser;
use crate::user::repository::user_manager_repository::UserManagerRepository;
use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};
use std::sync::Arc;

pub struct ListUserService {
    user_manager_repository: UserManagerRepository,
}

impl ListUserService {
    pub fn new(user_manager_repository: UserManagerRepository) -> Self {
        Self {
            user_manager_repository,
        }
    }

    pub fn list_users(&self) -> Arc<[ListUser]> {
        self.user_manager_repository
            .list_users()
            .unwrap_or_default()
    }
}

impl FromContext for ListUserService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
