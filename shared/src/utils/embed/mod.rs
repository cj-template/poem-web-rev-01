use poem::http::Uri;
use poem::{Endpoint, IntoEndpoint, Request};
use rust_embed::EmbeddedFile;

pub trait EmbedAsString {
    fn as_string(&self) -> String;
}

impl EmbedAsString for Option<EmbeddedFile> {
    fn as_string(&self) -> String {
        self.as_ref()
            .map(|f| String::from_utf8(f.data.to_vec()).unwrap_or_default())
            .unwrap_or_default()
    }
}

struct EnforceMinJsOnProd<EP: Endpoint>(EP);

impl<EP: Endpoint> Endpoint for EnforceMinJsOnProd<EP> {
    type Output = EP::Output;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        if cfg!(debug_assertions)
            || !req.uri().path().starts_with("/js/")
            || req.uri().path().to_lowercase().ends_with(".min.js")
            || !req.uri().path().to_lowercase().ends_with(".js")
        {
            return self.0.call(req).await;
        }

        {
            let uri = req.uri().clone();
            let mut uri_parts = uri.into_parts();
            if let Some(path_and_query) = uri_parts.path_and_query.as_ref() {
                let mut path = path_and_query.path().to_string();
                if let Some(new_path) = path.strip_suffix(".js") {
                    path = format!("{}.min.js", new_path)
                } else if let Some(new_path) = path.strip_suffix(".JS") {
                    path = format!("{}.MIN.JS", new_path)
                }
                if let Some(query) = path_and_query.query() {
                    path = format!("{}?{}", path, query);
                }
                uri_parts.path_and_query = Some(path.parse().expect("parse"));
                if let Ok(new_uri) = Uri::from_parts(uri_parts) {
                    *req.uri_mut() = new_uri;
                }
            }
        }

        self.0.call(req).await
    }
}

pub fn enforce_min_js_on_prod<EP: IntoEndpoint>(ep: EP) -> impl Endpoint {
    EnforceMinJsOnProd(ep.into_endpoint())
}
