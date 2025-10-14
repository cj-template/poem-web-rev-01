use crate::user::form::add_user::AddUserValidated;
use crate::user::layer::password_layer::PasswordLayer;
use crate::user::repository::user_manager_repository::UserManagerRepository;
use cjtoolkit_structured_validator::types::username::IsUsernameTakenAsync;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};
use shared::error::ExtraResultExt;
use shared::password::Password;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AddUserServiceError {
    #[error("User already exists")]
    SubmitFailed,
    #[error("Password Hash Error")]
    PasswordHashError,
    #[error("Password Serialize Error")]
    PasswordSerializeError,
}

pub struct AddUserService {
    user_manager_repository: UserManagerRepository,
    password_layer: PasswordLayer,
}

impl AddUserService {
    pub fn new(
        user_manager_repository: UserManagerRepository,
        password_layer: PasswordLayer,
    ) -> Self {
        Self {
            user_manager_repository,
            password_layer,
        }
    }

    pub fn add_user_submit(
        &self,
        add_user_validated: &AddUserValidated,
    ) -> Result<(), Report<AddUserServiceError>> {
        self.user_manager_repository
            .add_user(
                add_user_validated.username.as_str().to_string(),
                self.hash_password(add_user_validated.password.as_str())?
                    .encode_to_msg_pack()
                    .change_context(AddUserServiceError::PasswordSerializeError)
                    .log_it()
                    .attach(StatusCode::INTERNAL_SERVER_ERROR)?,
                &add_user_validated.role,
            )
            .change_context(AddUserServiceError::SubmitFailed)?;
        Ok(())
    }

    fn hash_password(&self, password: &str) -> Result<Password, Report<AddUserServiceError>> {
        self.password_layer
            .hash_password(password)
            .change_context(AddUserServiceError::PasswordHashError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IsUsernameTakenAsync for AddUserService {
    async fn is_username_taken_async(&self, username: &str) -> bool {
        self.user_manager_repository
            .username_taken(username.to_string())
            .ok()
            .unwrap_or_default()
    }
}

impl FromContext for AddUserService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?, ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::user::repository::user_manager_repository::UserManagerRepositoryError;
    use mry::Any;
    use shared::password::PasswordError;

    #[test]
    fn test_add_user_success() {
        let add_user_validated = AddUserValidated::new_test_data();

        let mut user_manager_repository = UserManagerRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        password_layer
            .mock_hash_password(add_user_validated.password.as_str())
            .returns_once(Ok(Password::Version1 {
                argon2: add_user_validated.password.as_str().to_string(),
            }));

        user_manager_repository
            .mock_add_user(
                add_user_validated.username.as_str().to_string(),
                Any,
                add_user_validated.role.clone(),
            )
            .returns_once(Ok(()));

        let service = AddUserService::new(user_manager_repository, password_layer);
        let result = service.add_user_submit(&add_user_validated);
        assert!(result.is_ok());
    }

    #[test]
    fn test_add_user_hash_fail() {
        let add_user_validated = AddUserValidated::new_test_data();

        let user_manager_repository = UserManagerRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        password_layer
            .mock_hash_password(add_user_validated.password.as_str())
            .returns_once(Err(Report::new(PasswordError("Failed".to_string()))));

        let service = AddUserService::new(user_manager_repository, password_layer);
        let result = service.add_user_submit(&add_user_validated);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_user_db_fail() {
        let add_user_validated = AddUserValidated::new_test_data();

        let mut user_manager_repository = UserManagerRepository::new_mock();
        let mut password_layer = PasswordLayer::new_mock();

        password_layer
            .mock_hash_password(add_user_validated.password.as_str())
            .returns_once(Ok(Password::Version1 {
                argon2: add_user_validated.password.as_str().to_string(),
            }));

        user_manager_repository
            .mock_add_user(
                add_user_validated.username.as_str().to_string(),
                Any,
                add_user_validated.role.clone(),
            )
            .returns_once(Err(Report::new(UserManagerRepositoryError::QueryError)));

        let service = AddUserService::new(user_manager_repository, password_layer);
        let result = service.add_user_submit(&add_user_validated);
        assert!(result.is_err());
    }
}
