use oxcord_model::prelude::*;
use oxcord_rest::CacheHttp;

use super::Builder;
use crate::internal::prelude::*;

/// Builder for modifying a guild template.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild-template#modify-guild-template).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditGuildTemplate {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<Option<String>>,
}

impl EditGuildTemplate {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the name of the template.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets or clears the description of the template.
    pub fn description(mut self, description: impl Into<Option<String>>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[async_trait::async_trait]
impl Builder for EditGuildTemplate {
    type Context<'ctx> = (GuildId, &'ctx str);
    type Built = GuildTemplate;

    /// Modifies the guild template.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the request fails.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<GuildTemplate> {
        cache_http.http().edit_guild_template(ctx.0, ctx.1, &self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_guild_template_serialization() {
        let builder = EditGuildTemplate::new()
            .name("Updated Name")
            .description(Some("Updated Desc".to_string()));
        let json = serde_json::to_string(&builder).unwrap();
        assert_eq!(json, r#"{"name":"Updated Name","description":"Updated Desc"}"#);

        let clear_desc = EditGuildTemplate::new().description(None);
        let json_clear = serde_json::to_string(&clear_desc).unwrap();
        assert_eq!(json_clear, r#"{"description":null}"#);
    }
}
