use error_stack::Report;
use shared::context::{Context, ContextError, FromContext};
use shared::password::{Password, PasswordError, PasswordState};

#[mry::mry]
pub struct PasswordLayer {}

impl PasswordLayer {
    pub fn new() -> Self {
        Self {
            mry: Default::default(),
        }
    }
}

#[mry::mry]
impl PasswordLayer {
    pub fn verify_password(
        &self,
        password_hash: Box<[u8]>,
        password: &str,
    ) -> Result<PasswordState, Report<PasswordError>> {
        Password::verify_password(password_hash, password.to_string())
    }

    pub fn hash_password(&self, password: &str) -> Result<Password, Report<PasswordError>> {
        Password::hash_password(password.to_string())
    }
}

#[cfg(test)]
impl PasswordLayer {
    pub fn new_mock() -> Self {
        mry::new!(Self {})
    }
}

impl FromContext for PasswordLayer {
    async fn from_context(_ctx: &'_ Context<'_>) -> Result<Self, Report<ContextError>> {
        Ok(Self::new())
    }
}
