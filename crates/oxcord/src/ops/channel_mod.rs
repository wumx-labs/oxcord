#[cfg(feature = "model")]
use crate::http::CacheHttp;
use crate::model::prelude::*;
#[allow(async_fn_in_trait)]
pub trait ChannelExt: Sized {
    #[cfg(feature = "model")]
    /// Converts from [`Channel`] to `Option<GuildChannel>`.
    ///
    /// Converts `self` into an `Option<GuildChannel>`, consuming `self`, and discarding a
    /// [`PrivateChannel`] if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// match channel.guild() {
    ///     Some(guild_channel) => {
    ///         println!("It's a guild channel named {}!", guild_channel.name);
    ///     },
    ///     None => {
    ///         println!("It's not in a guild!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    fn guild(self) -> Option<GuildChannel>;
    #[cfg(feature = "model")]
    /// Converts from [`Channel`] to `Option<PrivateChannel>`.
    ///
    /// Converts `self` into an `Option<PrivateChannel>`, consuming `self`, and discarding a
    /// [`GuildChannel`], if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```rust,no_run
    /// # use oxcord::prelude::*;
    /// # use oxcord::model::channel::Channel;
    /// # fn run(channel: Channel) {
    /// #
    /// match channel.private() {
    ///     Some(private) => {
    ///         println!("It's a private channel with {}!", &private.recipient);
    ///     },
    ///     None => {
    ///         println!("It's not a private channel!");
    ///     },
    /// }
    /// # }
    /// ```
    #[must_use]
    fn private(self) -> Option<PrivateChannel>;
    #[cfg(feature = "model")]
    /// If this is a category channel, returns it.
    #[must_use]
    fn category(self) -> Option<GuildChannel>;
    #[cfg(feature = "model")]
    /// Deletes the inner channel.
    ///
    /// # Errors
    ///
    /// If the `cache` is enabled, returns [`ModelError::InvalidPermissions`], if the current user
    /// lacks permission.
    ///
    /// Otherwise will return [`Error::Http`] if the current user does not have permission.
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()>;
    #[cfg(feature = "model")]
    /// Determines if the channel is NSFW.
    #[must_use]
    #[cfg(feature = "model")]
    #[deprecated(
        since = "0.0.1",
        note = "Use the GuildChannel::nsfw field, as PrivateChannel is never NSFW"
    )]
    fn is_nsfw(&self) -> bool;
}
impl ChannelExt for Channel {
    #[cfg(feature = "model")]
    fn guild(self) -> Option<GuildChannel> {
        match self {
            Self::Guild(lock) => Some(lock),
            _ => None,
        }
    }
    #[cfg(feature = "model")]
    fn private(self) -> Option<PrivateChannel> {
        match self {
            Self::Private(lock) => Some(lock),
            _ => None,
        }
    }
    #[cfg(feature = "model")]
    fn category(self) -> Option<GuildChannel> {
        match self {
            Self::Guild(c) if c.kind == ChannelType::Category => Some(c),
            _ => None,
        }
    }
    #[cfg(feature = "model")]
    async fn delete(&self, cache_http: impl CacheHttp) -> Result<()> {
        match self {
            Self::Guild(public_channel) => {
                public_channel.delete(cache_http).await?;
            },
            Self::Private(private_channel) => {
                private_channel.delete(cache_http.http()).await?;
            },
            _ => {},
        }
        Ok(())
    }
    #[cfg(feature = "model")]
    #[inline]
    #[cfg(feature = "model")]
    fn is_nsfw(&self) -> bool {
        match self {
            #[allow(deprecated)]
            Self::Guild(channel) => channel.is_nsfw(),
            _ => false,
        }
    }
}
