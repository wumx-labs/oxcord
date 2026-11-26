use oxcord_model::prelude::*;
use oxcord_rest::CacheHttp;

use super::Builder;
use crate::internal::prelude::*;

/// Builder for creating a guild template.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild-template#create-guild-template).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateGuildTemplate {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl CreateGuildTemplate {
    /// Creates a new builder with the given template name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    /// Sets the name of the template.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the description of the template.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[async_trait::async_trait]
impl Builder for CreateGuildTemplate {
    type Context<'ctx> = GuildId;
    type Built = GuildTemplate;

    /// Creates the guild template.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the request fails.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<GuildTemplate> {
        cache_http.http().create_guild_template(ctx, &self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_guild_template_serialization() {
        let builder = CreateGuildTemplate::new("My Template").description("Template description");
        let json = serde_json::to_string(&builder).unwrap();
        assert_eq!(json, r#"{"name":"My Template","description":"Template description"}"#);
    }
}
