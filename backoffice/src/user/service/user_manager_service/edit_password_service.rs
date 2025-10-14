use crate::user::form::edit_password_manager::EditPasswordManagerValidated;
use crate::user::layer::password_layer::PasswordLayer;
use crate::user::model::user_manager_model::FetchUser;
use crate::user::repository::user_manager_repository::UserManagerRepository;
use error_stack::{Report, ResultExt};
use poem::http::StatusCode;
use shared::context::{Context, ContextError, FromContext};
use shared::error::ExtraResultExt;
use shared::password::Password;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditPasswordServiceError {
    #[error("User not found")]
    UserNotFound,
    #[error("Database error")]
    DbError,
    #[error("Password Hash Error")]
    PasswordHashError,
    #[error("Password Serialize Error")]
    PasswordSerializeError,
}

pub struct EditPasswordService {
    user_manager_repository: UserManagerRepository,
    password_layer: PasswordLayer,
}

impl EditPasswordService {
    pub fn new(
        user_manager_repository: UserManagerRepository,
        password_layer: PasswordLayer,
    ) -> Self {
        Self {
            user_manager_repository,
            password_layer,
        }
    }

    pub fn edit_password_submit(
        &self,
        user_id: i64,
        password: &EditPasswordManagerValidated,
    ) -> Result<(), Report<EditPasswordServiceError>> {
        self.user_manager_repository
            .edit_password(
                user_id,
                self.hash_password(password.password.as_str())?
                    .encode_to_msg_pack()
                    .change_context(EditPasswordServiceError::PasswordSerializeError)
                    .log_it()
                    .attach(StatusCode::INTERNAL_SERVER_ERROR)?,
            )
            .change_context(EditPasswordServiceError::DbError)?;

        Ok(())
    }

    pub fn fetch_user(&self, user_id: i64) -> Result<FetchUser, Report<EditPasswordServiceError>> {
        self.user_manager_repository
            .fetch_user(user_id)
            .change_context(EditPasswordServiceError::UserNotFound)?
            .ok_or_else(|| {
                Report::new(EditPasswordServiceError::UserNotFound).attach(StatusCode::NOT_FOUND)
            })
    }

    fn hash_password(&self, password: &str) -> Result<Password, Report<EditPasswordServiceError>> {
        self.password_layer
            .hash_password(password)
            .change_context(EditPasswordServiceError::PasswordHashError)
            .attach(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl FromContext for EditPasswordService {
    async fn from_context(ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new(ctx.inject().await?, ctx.inject().await?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod test_edit_password_submit {
        use super::*;
        use crate::user::repository::user_manager_repository::UserManagerRepositoryError;
        use mry::Any;
        use shared::password::PasswordError;

        #[test]
        fn test_submit_success() {
            let password = EditPasswordManagerValidated::new_test_data();
            let mut user_manager_repository = UserManagerRepository::new_mock();
            let mut password_layer = PasswordLayer::new_mock();

            password_layer
                .mock_hash_password(password.password.as_str())
                .returns_once(Ok(Password::Version1 {
                    argon2: password.password.as_str().to_string(),
                }));

            user_manager_repository
                .mock_edit_password(1, Any)
                .returns_once(Ok(()));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.edit_password_submit(1, &password);
            assert!(result.is_ok());
        }

        #[test]
        fn test_password_hash_fail() {
            let password = EditPasswordManagerValidated::new_test_data();
            let user_manager_repository = UserManagerRepository::new_mock();
            let mut password_layer = PasswordLayer::new_mock();

            password_layer
                .mock_hash_password(password.password.as_str())
                .returns_once(Err(Report::new(PasswordError("Failed".to_string()))));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.edit_password_submit(1, &password);
            assert!(result.is_err());
        }

        #[test]
        fn test_submit_fail() {
            let password = EditPasswordManagerValidated::new_test_data();
            let mut user_manager_repository = UserManagerRepository::new_mock();
            let mut password_layer = PasswordLayer::new_mock();

            password_layer
                .mock_hash_password(password.password.as_str())
                .returns_once(Ok(Password::Version1 {
                    argon2: password.password.as_str().to_string(),
                }));

            user_manager_repository
                .mock_edit_password(1, Any)
                .returns_once(Err(Report::new(UserManagerRepositoryError::QueryError)));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.edit_password_submit(1, &password);
            assert!(result.is_err());
        }
    }

    mod test_fetch_user {
        use super::*;
        use crate::user::repository::user_manager_repository::UserManagerRepositoryError;

        #[test]
        fn test_fetch_user_success() {
            let mut user_manager_repository = UserManagerRepository::new_mock();
            let password_layer = PasswordLayer::new_mock();
            user_manager_repository
                .mock_fetch_user(1)
                .returns_once(Ok(Some(FetchUser {
                    username: "username".to_string(),
                    role: Default::default(),
                })));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.fetch_user(1);
            assert!(result.is_ok());
        }

        #[test]
        fn test_fetch_user_not_found() {
            let mut user_manager_repository = UserManagerRepository::new_mock();
            let password_layer = PasswordLayer::new_mock();
            user_manager_repository
                .mock_fetch_user(1)
                .returns_once(Ok(None));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.fetch_user(1);
            assert!(result.is_err());
            let result = result.err().unwrap();
            let status_code = result.downcast_ref::<StatusCode>().unwrap();
            assert_eq!(*status_code, StatusCode::NOT_FOUND);
        }

        #[test]
        fn test_fetch_user_error() {
            let mut user_manager_repository = UserManagerRepository::new_mock();
            let password_layer = PasswordLayer::new_mock();
            user_manager_repository
                .mock_fetch_user(1)
                .returns_once(Err(Report::new(UserManagerRepositoryError::QueryError)));

            let service = EditPasswordService::new(user_manager_repository, password_layer);
            let result = service.fetch_user(1);
            assert!(result.is_err());
        }
    }
}
