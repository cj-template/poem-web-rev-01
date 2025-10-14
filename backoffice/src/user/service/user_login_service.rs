use crate::user::LOGIN_TOKEN_COOKIE_NAME;
use crate::user::layer::password_layer::PasswordLayer;
use crate::user::repository::user_repository::UserRepository;
use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};
use uuid::Uuid;

pub struct UserLoginService {
    user_repository: UserRepository,
    password_layer: PasswordLayer,
    token_cookie: Option<String>,
}

impl UserLoginService {
    pub fn new(
        user_repository: UserRepository,
        password_layer: PasswordLayer,
        token_cookie: Option<String>,
    ) -> Self {
        Self {
            user_repository,
            password_layer,
            token_cookie,
        }
    }

    pub fn validate_login(&self, username: String, password: String) -> Option<String> {
        if let Ok(id_password) = self.user_repository.get_user_password(username) {
            let password_status = self
                .password_layer
                .verify_password(id_password.password, password.as_str());
            if let Ok(password_state) = password_status {
                if password_state.is_valid() {
                    let uuid = Uuid::new_v4().to_string();

                    if self
                        .user_repository
                        .add_token(uuid.clone(), id_password.id)
                        .is_err()
                    {
                        return None;
                    }

                    return Some(uuid);
                }
            }
        }
        None
    }

    pub fn logout(&self) -> bool {
        if let Some(token) = self.token_cookie.as_ref() {
            self.user_repository.delete_token(token.to_string()).is_ok()
        } else {
            false
        }
    }
}

impl FromContext for UserLoginService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        let req = ctx.req_result()?;
        let cookie = req.cookie();
        Ok(Self::new(
            ctx.inject().await?,
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
    use crate::user::model::user_model::IdPassword;
    use crate::user::repository::user_repository::UserRepositoryError;
    use mry::Any;
    use shared::password::PasswordState;

    #[test]
    fn test_validate_login_success() {
        let mut user_repository = UserRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_get_user_password("hello".to_string())
            .returns_once(Ok(IdPassword {
                id: 1,
                password: Default::default(),
            }));

        password_layer
            .mock_verify_password(Any, "password")
            .returns_once(Ok(PasswordState::Valid));

        user_repository.mock_add_token(Any, 1).returns_once(Ok(()));

        let service = UserLoginService::new(user_repository, password_layer, None);
        let str = service.validate_login("hello".to_string(), "password".to_string());
        assert!(str.is_some());
    }

    #[test]
    fn test_validate_login_username_error() {
        let mut user_repository = UserRepository::new_mock();
        let password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_get_user_password("hello".to_string())
            .returns_once(Err(Report::new(UserRepositoryError::QueryError)));

        let service = UserLoginService::new(user_repository, password_layer, None);
        let str = service.validate_login("hello".to_string(), "password".to_string());
        assert!(str.is_none());
    }

    #[test]
    fn test_validate_login_password_verify_fail() {
        let mut user_repository = UserRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_get_user_password("hello".to_string())
            .returns_once(Ok(IdPassword {
                id: 1,
                password: Default::default(),
            }));

        password_layer
            .mock_verify_password(Any, "password")
            .returns_once(Ok(PasswordState::Invalid));

        let service = UserLoginService::new(user_repository, password_layer, None);
        let str = service.validate_login("hello".to_string(), "password".to_string());
        assert!(str.is_none());
    }

    #[test]
    fn test_validate_login_add_token_fail() {
        let mut user_repository = UserRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_get_user_password("hello".to_string())
            .returns_once(Ok(IdPassword {
                id: 1,
                password: Default::default(),
            }));

        password_layer
            .mock_verify_password(Any, "password")
            .returns_once(Ok(PasswordState::Valid));

        user_repository
            .mock_add_token(Any, 1)
            .returns_once(Err(Report::new(UserRepositoryError::QueryError)));

        let service = UserLoginService::new(user_repository, password_layer, None);
        let str = service.validate_login("hello".to_string(), "password".to_string());
        assert!(str.is_none());
    }

    #[test]
    fn test_logout_success() {
        let mut user_repository = UserRepository::new_mock();
        let password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_delete_token("hello".to_string())
            .returns_once(Ok(()));

        let service =
            UserLoginService::new(user_repository, password_layer, Some("hello".to_string()));
        let result = service.logout();
        assert_eq!(result, true);
    }

    #[test]
    fn test_logout_fail() {
        let mut user_repository = UserRepository::new_mock();
        let password_layer = PasswordLayer::new_mock();

        user_repository
            .mock_delete_token("hello".to_string())
            .returns_once(Err(Report::new(UserRepositoryError::QueryError)));

        let service =
            UserLoginService::new(user_repository, password_layer, Some("hello".to_string()));
        let result = service.logout();
        assert_eq!(result, false);
    }
}
