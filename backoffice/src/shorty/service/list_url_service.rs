use crate::shorty::model::shorty_model::ListUrlRedirectModel;
use crate::shorty::repository::shorty_repository::ShortyRepository;
use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};
use std::sync::Arc;

pub struct ListUrlService {
    shorty_repository: ShortyRepository,
}

impl ListUrlService {
    pub fn new(shorty_repository: ShortyRepository) -> Self {
        Self { shorty_repository }
    }

    pub fn list_urls(&self) -> Arc<[ListUrlRedirectModel]> {
        self.shorty_repository
            .list_url_redirect()
            .unwrap_or_default()
    }
}

impl FromContext for ListUrlService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?))
    }
}
