//! The HTTP module re-exported from `oxcord_rest`.

#[cfg(feature = "cache")]
use std::sync::Arc;

pub use oxcord_rest::*;

#[cfg(feature = "cache")]
use crate::cache::Cache;
#[cfg(feature = "client")]
use crate::client::Context;

#[cfg(feature = "client")]
impl CacheHttp for Context {
    fn http(&self) -> &Http {
        &self.http
    }
    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&Arc<Cache>> {
        Some(&self.cache)
    }
}
