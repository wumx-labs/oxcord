use crate::model::prelude::*;

#[cfg(feature = "model")]
impl From<Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: Channel) -> ChannelId {
        channel.id()
    }
}

#[cfg(feature = "model")]
impl From<&Channel> for ChannelId {
    /// Gets the Id of a [`Channel`].
    fn from(channel: &Channel) -> ChannelId {
        channel.id()
    }
}

impl From<PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<&PrivateChannel> for ChannelId {
    /// Gets the Id of a private channel.
    fn from(private_channel: &PrivateChannel) -> ChannelId {
        private_channel.id
    }
}

impl From<GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<&GuildChannel> for ChannelId {
    /// Gets the Id of a guild channel.
    fn from(public_channel: &GuildChannel) -> ChannelId {
        public_channel.id
    }
}

impl From<WebhookChannel> for ChannelId {
    /// Gets the Id of a webhook channel.
    fn from(webhook_channel: WebhookChannel) -> ChannelId {
        webhook_channel.id
    }
}

impl From<&WebhookChannel> for ChannelId {
    /// Gets the Id of a webhook channel.
    fn from(webhook_channel: &WebhookChannel) -> ChannelId {
        webhook_channel.id
    }
}
