use poem::{Endpoint, IntoEndpoint, Request};
use serde_qs::Config;

struct SerdeQsConfig<E: Endpoint>(Config, E);

impl<E: Endpoint> Endpoint for SerdeQsConfig<E> {
    type Output = E::Output;

    async fn call(&self, mut req: Request) -> poem::Result<Self::Output> {
        req.set_data(self.0.clone());

        self.1.call(req).await
    }
}

pub fn with_serde_qs_config<E>(config: Config, endpoint: E) -> impl Endpoint
where
    E: IntoEndpoint,
    E::Endpoint: 'static,
{
    SerdeQsConfig(config, endpoint.into_endpoint())
}
