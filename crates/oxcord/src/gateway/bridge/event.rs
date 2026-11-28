//! A collection of events created by the client, not a part of the Discord API itself.

use crate::gateway::ConnectionStage;
use crate::model::id::ShardId;

/// An event denoting that a shard's connection stage was changed.
///
/// # Examples
///
/// This might happen when a shard changes from [`ConnectionStage::Identifying`] to
/// [`ConnectionStage::Connected`].
#[derive(Clone, Debug)]
pub struct ShardStageUpdateEvent {
    /// The new connection stage.
    pub new: ConnectionStage,
    /// The old connection stage.
    pub old: ConnectionStage,
    /// The ID of the shard that had its connection stage change.
    pub shard_id: ShardId,
    /// The websocket close code the gateway sent, when this update follows a disconnect with a
    /// close frame. `None` otherwise.
    ///
    /// When it is `Some`, the event may have `new == old`: the shard was told to close but the
    /// reconnect has not started yet.
    pub close_code: Option<u16>,
    /// The close reason that came with [`Self::close_code`], possibly empty.
    pub close_reason: Option<String>,
}
