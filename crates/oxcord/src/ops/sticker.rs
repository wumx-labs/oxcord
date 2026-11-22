#[cfg(feature = "model")]
use crate::builder::EditSticker;
#[cfg(feature = "model")]
use crate::http::{CacheHttp, Http};
use crate::model::prelude::*;
#[cfg(feature = "model")]
use crate::model::sticker::banner_url;
#[cfg(feature = "model")]
use crate::model::sticker::sticker_url;
#[allow(async_fn_in_trait)]
pub trait StickerExt: Sized {
    #[cfg(feature = "model")]
    /// Deletes the [`Sticker`] from its guild.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission to delete the sticker.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Edits the sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Examples
    ///
    /// Rename a sticker:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::http::Http;
    /// # use oxcord::model::id::GuildId;
    /// # use oxcord::model::sticker::Sticker;
    /// use oxcord::builder::EditSticker;
    ///
    /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
    /// # let http: Http = unimplemented!();
    /// # let mut sticker: Sticker = unimplemented!();
    /// let builder = EditSticker::new().name("Bun bun meow");
    /// sticker.edit(&http, builder).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditSticker<'_>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Retrieves the URL to the sticker image.
    ///
    /// **Note**: This will only be `None` if the format_type is unknown.
    #[must_use]
    fn image_url(&self) -> Option<String>;
}
impl StickerExt for Sticker {
    #[cfg(feature = "model")]
    #[inline]
    async fn delete(&self, http: impl AsRef<Http>) -> Result<()> {
        if let Some(guild_id) = self.guild_id {
            guild_id.delete_sticker(http, self.id).await
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }
    #[cfg(feature = "model")]
    #[inline]
    async fn edit(&mut self, cache_http: impl CacheHttp, builder: EditSticker<'_>) -> Result<()> {
        if let Some(guild_id) = self.guild_id {
            *self = guild_id.edit_sticker(cache_http, self.id, builder).await?;
            Ok(())
        } else {
            Err(Error::Model(ModelError::DeleteNitroSticker))
        }
    }
    #[cfg(feature = "model")]
    #[inline]
    fn image_url(&self) -> Option<String> {
        sticker_url(self.id, self.format_type)
    }
}
#[allow(async_fn_in_trait)]
pub trait StickerIdExt: Sized {
    #[cfg(feature = "model")]
    /// Delete a guild sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    #[deprecated(since = "0.0.1", note = "use `GuildId::delete_sticker` instead")]
    async fn delete(self, http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<()>;
    #[cfg(feature = "model")]
    /// Requests the sticker via the REST API to get a [`Sticker`] with all details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does not exist, or is
    /// otherwise unavailable.
    async fn to_sticker(self, http: impl AsRef<Http>) -> Result<Sticker>;
    #[cfg(feature = "model")]
    /// Edits the sticker.
    ///
    /// **Note**: If the sticker was created by the current user, requires either the [Create Guild
    /// Expressions] or the [Manage Guild Expressions] permission. Otherwise, the [Manage Guild
    /// Expressions] permission is required.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the current user lacks permission, or if invalid data is given.
    ///
    /// [Create Guild Expressions]: Permissions::CREATE_GUILD_EXPRESSIONS
    /// [Manage Guild Expressions]: Permissions::MANAGE_GUILD_EXPRESSIONS
    #[deprecated(since = "0.0.1", note = "use `GuildId::edit_sticker` instead")]
    async fn edit(
        self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker>;
}
impl StickerIdExt for StickerId {
    #[cfg(feature = "model")]
    async fn delete(self, http: impl AsRef<Http>, guild_id: impl Into<GuildId>) -> Result<()> {
        guild_id.into().delete_sticker(http, self).await
    }
    #[cfg(feature = "model")]
    async fn to_sticker(self, http: impl AsRef<Http>) -> Result<Sticker> {
        http.as_ref().get_sticker(self).await.map_err(From::from)
    }
    #[cfg(feature = "model")]
    async fn edit(
        self,
        cache_http: impl CacheHttp,
        guild_id: impl Into<GuildId>,
        builder: EditSticker<'_>,
    ) -> Result<Sticker> {
        guild_id.into().edit_sticker(cache_http, self, builder).await
    }
}
#[allow(async_fn_in_trait)]
pub trait StickerItemExt: Sized {
    #[cfg(feature = "model")]
    /// Requests the sticker via the REST API to get a [`Sticker`] with all details.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`Sticker`] with that [`StickerId`] does
    /// not exist, or is otherwise unavailable.
    async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker>;
    #[cfg(feature = "model")]
    /// Retrieves the URL to the sticker image.
    ///
    /// **Note**: This will only be `None` if the format_type is unknown.
    #[must_use]
    fn image_url(&self) -> Option<String>;
}
impl StickerItemExt for StickerItem {
    #[cfg(feature = "model")]
    #[inline]
    async fn to_sticker(&self, http: impl AsRef<Http>) -> Result<Sticker> {
        self.id.to_sticker(http).await
    }
    #[cfg(feature = "model")]
    #[inline]
    fn image_url(&self) -> Option<String> {
        sticker_url(self.id, self.format_type)
    }
}
#[allow(async_fn_in_trait)]
pub trait StickerPackExt: Sized {
    #[cfg(feature = "model")]
    /// Returns the sticker that is shown as the pack's icon
    #[must_use]
    fn cover_sticker(&self) -> Option<&Sticker>;
    #[cfg(feature = "model")]
    #[must_use]
    fn banner_url(&self) -> String;
}
impl StickerPackExt for StickerPack {
    #[cfg(feature = "model")]
    fn cover_sticker(&self) -> Option<&Sticker> {
        self.cover_sticker_id.and_then(|id| self.stickers.iter().find(|s| s.id == id))
    }
    #[cfg(feature = "model")]
    fn banner_url(&self) -> String {
        banner_url(self.banner_asset_id)
    }
}
#[allow(async_fn_in_trait)]
pub trait StickerPackIdExt: Sized {
    #[cfg(feature = "model")]
    /// Gets the [`StickerPack`] object.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if a [`StickerPack`] with that [`StickerPackId`] does not exist, or
    /// is otherwise unavailable.
    async fn to_sticker_pack(self, http: impl AsRef<Http>) -> Result<StickerPack>;
}
impl StickerPackIdExt for StickerPackId {
    #[cfg(feature = "model")]
    async fn to_sticker_pack(self, http: impl AsRef<Http>) -> Result<StickerPack> {
        http.as_ref().get_sticker_pack(self).await.map_err(From::from)
    }
}
