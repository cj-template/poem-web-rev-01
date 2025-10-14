use poem::web::Path;
use poem::{FromRequest, Request, RequestBody};
use serde::de::DeserializeOwned;
use std::ops::{Deref, DerefMut};
use crate::flag::Flag;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct PathEdit<T: Default + DeserializeOwned>(pub T);

impl<T: Default + DeserializeOwned> Deref for PathEdit<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Default + DeserializeOwned> DerefMut for PathEdit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: Default + DeserializeOwned> FromRequest<'a> for PathEdit<T> {
    async fn from_request(req: &'a Request, _body: &mut RequestBody) -> poem::Result<Self> {
        let edit = req.data::<Flag>().map(|flag| flag.is_edit()).unwrap_or(false);
        if edit {
            let path = Path::<T>::from_request_without_body(req).await?;
            return Ok(Self(path.0));
        }
        Ok(Self(T::default()))
    }
}
