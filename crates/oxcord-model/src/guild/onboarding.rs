use crate::model::prelude::*;

/// Guild Onboarding object.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Onboarding {
    /// ID of the guild this onboarding is part of.
    pub guild_id: GuildId,
    /// Prompts shown during onboarding and in customize community.
    pub prompts: Vec<OnboardingPrompt>,
    /// Channel IDs that members get opted into after onboarding.
    pub default_channel_ids: Vec<ChannelId>,
    /// Whether onboarding is enabled in the guild.
    pub enabled: bool,
    /// Current mode of onboarding.
    pub mode: OnboardingMode,
}

enum_number! {
    /// Defines the mode of onboarding.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-mode).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingMode {
        /// Counts only Default Channels towards constraints.
        Default = 0,
        /// Counts Default Channels and Questions towards constraints.
        Advanced = 1,
        _ => Unknown(u8),
    }
}

enum_number! {
    /// Defines the type of onboarding prompt.
    ///
    /// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-prompt-types).
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum OnboardingPromptType {
        /// Multiple choice with single select.
        MultipleChoice = 0,
        /// Dropdown with single or multi select.
        Dropdown = 1,
        _ => Unknown(u8),
    }
}

/// Prompt shown during onboarding.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-prompt-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OnboardingPrompt {
    /// ID of the prompt.
    pub id: OnboardingPromptId,
    /// Type of the prompt.
    #[serde(rename = "type")]
    pub kind: OnboardingPromptType,
    /// Options available for this prompt.
    pub options: Vec<OnboardingPromptOption>,
    /// Title of the prompt.
    pub title: String,
    /// Indicates whether users are required to answer this prompt before completing onboarding.
    pub single_select: bool,
    /// Indicates whether users are required to answer this prompt before completing onboarding.
    pub required: bool,
    /// Indicates whether the prompt is present in the onboarding flow. If false, it will only
    /// appear in the Channels & Roles tab.
    pub in_onboarding: bool,
}

/// Option for an onboarding prompt.
///
/// [Discord docs](https://discord.com/developers/docs/resources/guild#guild-onboarding-object-onboarding-prompt-option-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OnboardingPromptOption {
    /// ID of the option.
    pub id: OnboardingOptionId,
    /// IDs for channels a member is added to when the option is selected.
    pub channel_ids: Vec<ChannelId>,
    /// IDs for roles assigned to a member when the option is selected.
    pub role_ids: Vec<RoleId>,
    /// Emoji of the option.
    pub emoji: Option<Emoji>,
    /// Title of the option.
    pub title: String,
    /// Description of the option.
    pub description: Option<String>,
}
