#[cfg(feature = "model")]
use crate::builder::{Builder as _, GetEntitlements};
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait EmojiIdentifierExt: Sized {
    #[cfg(all(feature = "model", feature = "utils"))]
    /// Generates a URL to the emoji's image.
    #[must_use]
    fn url(&self) -> String;
}
impl EmojiIdentifierExt for EmojiIdentifier {
    #[cfg(all(feature = "model", feature = "utils"))]
    fn url(&self) -> String {
        let ext = if self.animated { "gif" } else { "png" };
        cdn!("/emojis/{}.{}", self.id, ext)
    }
}

#[cfg(feature = "model")]
#[allow(async_fn_in_trait)]
pub trait EntitlementExt {
    /// For a one-time purchase consumable SKU (of kind [`Consumable`]), marks the entitlement as
    /// consumed. On success, the [`consumed`] field will be set to `Some(true)`.
    ///
    /// # Errors
    ///
    /// Will fail if the corresponding SKU is not of kind [`Consumable`].
    ///
    /// [`Consumable`]: SkuKind::Consumable
    /// [`consumed`]: Entitlement::consumed
    #[cfg(feature = "model")]
    async fn consume(&mut self, http: &Http) -> Result<()>;

    /// Returns all entitlements for the current application, active and expired.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "model")]
    async fn list(cache_http: impl CacheHttp, builder: GetEntitlements)
    -> Result<Vec<Entitlement>>;
}

#[cfg(feature = "model")]
impl EntitlementExt for Entitlement {
    /// For a one-time purchase consumable SKU (of kind [`Consumable`]), marks the entitlement as
    /// consumed. On success, the [`consumed`] field will be set to `Some(true)`.
    ///
    /// # Errors
    ///
    /// Will fail if the corresponding SKU is not of kind [`Consumable`].
    ///
    /// [`Consumable`]: SkuKind::Consumable
    /// [`consumed`]: Entitlement::consumed
    #[cfg(feature = "model")]
    async fn consume(&mut self, http: &Http) -> Result<()> {
        http.consume_entitlement(self.id).await?;
        self.consumed = Some(true);
        Ok(())
    }

    /// Returns all entitlements for the current application, active and expired.
    ///
    /// # Errors
    ///
    /// May error due to an invalid response from discord, or network error.
    #[cfg(feature = "model")]
    async fn list(
        cache_http: impl CacheHttp,
        builder: GetEntitlements,
    ) -> Result<Vec<Entitlement>> {
        builder.execute(cache_http, ()).await.map_err(From::from)
    }
}
