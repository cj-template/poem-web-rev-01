use poem::Endpoint;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

struct RequestCache(Arc<Mutex<HashMap<TypeId, Box<dyn Any + Send + Sync>>>>);

impl RequestCache {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(HashMap::new())))
    }

    async fn get_or_init_cache<T, F, Fut, E>(&self, f: F) -> Result<T, E>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let mut req_cache = self.0.lock().await;
        let type_id = TypeId::of::<T>();
        if let Some(cache) = req_cache.get(&type_id) {
            if let Some(cache) = cache.downcast_ref::<T>() {
                return Ok(cache.clone());
            }
        }
        let cache = f().await?;
        req_cache.insert(type_id, Box::new(cache.clone()));
        Ok(cache)
    }
}

impl Clone for RequestCache {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

pub async fn init_request_cache<EP: Endpoint>(
    next: EP,
    mut req: poem::Request,
) -> poem::Result<EP::Output> {
    req.set_data(RequestCache::new());
    next.call(req).await
}

pub trait RequestCacheExt {
    fn get_or_init_cache<T, F, Fut, E>(&self, f: F) -> impl Future<Output = Result<T, E>>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>;
}

impl RequestCacheExt for poem::Request {
    async fn get_or_init_cache<T, F, Fut, E>(&self, f: F) -> Result<T, E>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let req_cache = self
            .data::<RequestCache>()
            .expect("Request Cache not found");
        req_cache.get_or_init_cache(f).await
    }
}
