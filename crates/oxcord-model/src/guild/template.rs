use crate::model::prelude::*;

/// A template for creating a new guild based on an existing one.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild-template#guild-template-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GuildTemplate {
    /// The template code (unique ID).
    pub code: String,
    /// The name of the template.
    pub name: String,
    /// The description of the template.
    pub description: Option<String>,
    /// Number of times this template has been used.
    pub usage_count: u64,
    /// The ID of the user who created the template.
    pub creator_id: UserId,
    /// The user who created the template.
    pub creator: User,
    /// When the template was created.
    pub created_at: Timestamp,
    /// When the template was last synced to the source guild.
    pub updated_at: Timestamp,
    /// The ID of the guild the template is based on.
    pub source_guild_id: GuildId,
    /// The guild snapshot this template contains.
    pub serialized_source_guild: PartialGuild,
    /// Whether the template has un-synced changes.
    pub is_dirty: Option<bool>,
}
