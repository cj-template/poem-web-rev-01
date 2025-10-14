use crate::user::LOGIN_TOKEN_COOKIE_NAME;
use crate::user::model::user_model::UserIdContext;
use crate::user::repository::user_repository::UserRepository;
use crate::user::role::Role;
use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};

pub struct UserCheckService {
    user_repository: UserRepository,
    token_cookie: Option<String>,
}

impl UserCheckService {
    pub fn new(user_repository: UserRepository, token_cookie: Option<String>) -> Self {
        Self {
            user_repository,
            token_cookie,
        }
    }

    pub fn get_user_context(&self) -> UserIdContext {
        if let Some(user_context) = self.is_logged_in() {
            user_context
        } else {
            UserIdContext {
                id: 0,
                username: "visitor".to_string(),
                role: Role::Visitor,
            }
        }
    }

    fn is_logged_in(&self) -> Option<UserIdContext> {
        if let Some(token) = self.token_cookie.as_ref() {
            self.user_repository.find_by_token(token.to_string()).ok()
        } else {
            None
        }
    }
}

impl FromContext for UserCheckService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let req = ctx.req_result()?;
        let cookie = req.cookie();
        Ok(Self::new(
            ctx.inject().await?,
            cookie
                .get(LOGIN_TOKEN_COOKIE_NAME)
                .map(|v| v.value_str().to_string()),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::repository::user_repository::UserRepositoryError;

    #[test]
    fn test_get_user_context_user() {
        let mut user_repository = UserRepository::new_mock();

        user_repository
            .mock_find_by_token("hello".to_string())
            .returns_once(Ok(UserIdContext {
                id: 5,
                username: "".to_string(),
                role: Default::default(),
            }));

        let service = UserCheckService::new(user_repository, Some("hello".to_string()));
        let result = service.get_user_context();
        assert_eq!(result.id, 5);
    }

    #[test]
    fn test_get_user_context_visitor() {
        let mut user_repository = UserRepository::new_mock();

        user_repository
            .mock_find_by_token("hello".to_string())
            .returns_once(Err(Report::new(UserRepositoryError::QueryError)));

        let service = UserCheckService::new(user_repository, Some("hello".to_string()));
        let result = service.get_user_context();
        assert_eq!(result.id, 0);
    }
}
