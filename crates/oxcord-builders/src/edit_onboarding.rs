use oxcord_model::prelude::*;
use oxcord_rest::CacheHttp;

use super::Builder;
use crate::internal::prelude::*;

/// Builder for modifying guild onboarding.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#modify-guild-onboarding).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct EditOnboarding {
    #[serde(skip_serializing_if = "Option::is_none")]
    prompts: Option<Vec<CreateOnboardingPrompt>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_channel_ids: Option<Vec<ChannelId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<OnboardingMode>,
}

impl EditOnboarding {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the prompts shown during onboarding and in customize community.
    pub fn prompts(mut self, prompts: impl Into<Vec<CreateOnboardingPrompt>>) -> Self {
        self.prompts = Some(prompts.into());
        self
    }

    /// Adds a prompt to onboarding.
    pub fn add_prompt(mut self, prompt: CreateOnboardingPrompt) -> Self {
        self.prompts.get_or_insert_with(Vec::new).push(prompt);
        self
    }

    /// Sets the channel IDs that members get opted into after onboarding.
    pub fn default_channel_ids(mut self, channel_ids: impl Into<Vec<ChannelId>>) -> Self {
        self.default_channel_ids = Some(channel_ids.into());
        self
    }

    /// Sets whether onboarding is enabled in the guild.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the mode of onboarding.
    pub fn mode(mut self, mode: OnboardingMode) -> Self {
        self.mode = Some(mode);
        self
    }
}

#[async_trait::async_trait]
impl Builder for EditOnboarding {
    type Context<'ctx> = GuildId;
    type Built = Onboarding;

    /// Modifies the guild onboarding configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Http`] if the request fails.
    async fn execute(
        self,
        cache_http: impl CacheHttp,
        ctx: Self::Context<'_>,
    ) -> Result<Onboarding> {
        cache_http.http().edit_guild_onboarding(ctx, &self).await
    }
}

/// Builder for creating or updating an onboarding prompt.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-prompt-structure).
#[derive(Clone, Debug, Serialize)]
#[must_use]
pub struct CreateOnboardingPrompt {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<OnboardingPromptId>,
    #[serde(rename = "type")]
    pub kind: OnboardingPromptType,
    pub options: Vec<CreateOnboardingPromptOption>,
    pub title: String,
    pub single_select: bool,
    pub required: bool,
    pub in_onboarding: bool,
}

impl CreateOnboardingPrompt {
    /// Creates a new prompt builder with required fields.
    pub fn new(
        title: impl Into<String>,
        kind: OnboardingPromptType,
        options: impl Into<Vec<CreateOnboardingPromptOption>>,
    ) -> Self {
        Self {
            id: None,
            kind,
            options: options.into(),
            title: title.into(),
            single_select: false,
            required: false,
            in_onboarding: true,
        }
    }

    /// Sets the ID of the prompt (to update an existing prompt).
    pub fn id(mut self, id: impl Into<OnboardingPromptId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets whether users can only select a single option.
    pub fn single_select(mut self, single_select: bool) -> Self {
        self.single_select = single_select;
        self
    }

    /// Sets whether answering this prompt is required before completing onboarding.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets whether the prompt is present in the onboarding flow.
    pub fn in_onboarding(mut self, in_onboarding: bool) -> Self {
        self.in_onboarding = in_onboarding;
        self
    }
}

/// Builder for creating or updating an onboarding prompt option.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-prompt-option-structure).
#[derive(Clone, Debug, Default, Serialize)]
#[must_use]
pub struct CreateOnboardingPromptOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<OnboardingOptionId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub channel_ids: Vec<ChannelId>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub role_ids: Vec<RoleId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<Emoji>,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateOnboardingPromptOption {
    /// Creates a new prompt option builder.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            id: None,
            channel_ids: Vec::new(),
            role_ids: Vec::new(),
            emoji: None,
            title: title.into(),
            description: None,
        }
    }

    /// Sets the ID of the option (when updating an existing option).
    pub fn id(mut self, id: impl Into<OnboardingOptionId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the channel IDs a member is added to when selected.
    pub fn channel_ids(mut self, channel_ids: impl Into<Vec<ChannelId>>) -> Self {
        self.channel_ids = channel_ids.into();
        self
    }

    /// Sets the role IDs assigned to a member when selected.
    pub fn role_ids(mut self, role_ids: impl Into<Vec<RoleId>>) -> Self {
        self.role_ids = role_ids.into();
        self
    }

    /// Sets the emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<Option<Emoji>>) -> Self {
        self.emoji = emoji.into();
        self
    }

    /// Sets the description of the option.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edit_onboarding_serialization() {
        let opt = CreateOnboardingPromptOption::new("Rust Dev")
            .channel_ids(vec![ChannelId::new(100)])
            .role_ids(vec![RoleId::new(200)])
            .description("For Rustaceans");

        let prompt = CreateOnboardingPrompt::new(
            "Choose your role",
            OnboardingPromptType::MultipleChoice,
            vec![opt],
        )
        .single_select(true)
        .required(true);

        let builder = EditOnboarding::new()
            .enabled(true)
            .mode(OnboardingMode::Advanced)
            .default_channel_ids(vec![ChannelId::new(101)])
            .add_prompt(prompt);

        let json = serde_json::to_string(&builder).unwrap();
        assert!(json.contains(r#""enabled":true"#));
        assert!(json.contains(r#""mode":1"#));
        assert!(json.contains(r#""title":"Choose your role""#));
        assert!(json.contains(r#""title":"Rust Dev""#));
    }
}
