use crate::user::pointer::user_pointer::UserPointer;
use crate::user::role::Role;
use poem::http::StatusCode;
use poem::{Endpoint, Error, FromRequest, IntoEndpoint, Request};
use shared::context::Dep;

struct UserRoleCheck<E: Endpoint>(Role, E);

impl<E: Endpoint> Endpoint for UserRoleCheck<E> {
    type Output = E::Output;

    async fn call(&self, req: Request) -> poem::Result<Self::Output> {
        let Dep(user_context) = Dep::<UserPointer>::from_request_without_body(&req).await?;

        if user_context.role < self.0 {
            return Err(Error::from_status(StatusCode::UNAUTHORIZED));
        }

        self.1.call(req).await
    }
}

pub fn must_be_user<E>(endpoint: E) -> impl Endpoint
where
    E: IntoEndpoint,
    E::Endpoint: 'static,
{
    UserRoleCheck(Role::User, endpoint.into_endpoint())
}

pub fn must_be_root<E>(endpoint: E) -> impl Endpoint
where
    E: IntoEndpoint,
    E::Endpoint: 'static,
{
    UserRoleCheck(Role::Root, endpoint.into_endpoint())
}
