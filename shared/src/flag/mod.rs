pub mod path_edit;

use poem::{Endpoint, FromRequest, IntoEndpoint, Request, RequestBody};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Add,
    Edit,
    Delete,
}

impl Default for Flag {
    fn default() -> Self {
        Self::Add
    }
}

impl Flag {
    pub fn is_add(&self) -> bool {
        *self == Self::Add
    }

    pub fn is_edit(&self) -> bool {
        *self == Self::Edit
    }

    pub fn is_delete(&self) -> bool {
        *self == Self::Delete
    }
}

impl<'a> FromRequest<'a> for Flag {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        match req.data::<Flag>() {
            None => Ok(Flag::default()),
            Some(flag) => Ok(*flag),
        }
    }
}

struct Flagger<EP: Endpoint>(Flag, EP);

impl<EP: Endpoint> Endpoint for Flagger<EP> {
    type Output = EP::Output;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        req.set_data(self.0);
        self.1.call(req).await
    }
}

pub fn flag_add<EP: IntoEndpoint>(ep: EP) -> impl Endpoint {
    Flagger(Flag::Add, ep.into_endpoint())
}

pub fn flag_edit<EP: IntoEndpoint>(ep: EP) -> impl Endpoint {
    Flagger(Flag::Edit, ep.into_endpoint())
}

pub fn flag_delete<EP: IntoEndpoint>(ep: EP) -> impl Endpoint {
    Flagger(Flag::Delete, ep.into_endpoint())
}
