use serde::de::Error as DeError;
use serde::ser::{Serialize, Serializer};

use crate::internal::prelude::*;
use crate::json::from_value;
use crate::model::prelude::*;
use crate::model::utils::{default_true, deserialize_val};

enum_number! {
    /// The type of a component
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ComponentType {
        ActionRow = 1,
        Button = 2,
        StringSelect = 3,
        InputText = 4,
        UserSelect = 5,
        RoleSelect = 6,
        MentionableSelect = 7,
        ChannelSelect = 8,
        Section = 9,
        TextDisplay = 10,
        Thumbnail = 11,
        MediaGallery = 12,
        File = 13,
        Separator = 14,
        Container = 15,
        Label = 16,
        _ => Unknown(u8),
    }
}

/// An action row.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ActionRow {
    /// Always [`ComponentType::ActionRow`]
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The components of this ActionRow.
    #[serde(default)]
    pub components: Vec<ActionRowComponent>,
}

/// A component which can be inside of an [`ActionRow`].
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row-action-row-child-components).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ActionRowComponent {
    Button(Button),
    SelectMenu(SelectMenu),
    InputText(InputText),
}

impl<'de> Deserialize<'de> for ActionRowComponent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;

        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::Button => from_value(value).map(ActionRowComponent::Button),
            ComponentType::InputText => from_value(value).map(ActionRowComponent::InputText),
            ComponentType::StringSelect
            | ComponentType::UserSelect
            | ComponentType::RoleSelect
            | ComponentType::MentionableSelect
            | ComponentType::ChannelSelect => from_value(value).map(ActionRowComponent::SelectMenu),
            ComponentType::ActionRow => {
                return Err(DeError::custom("Invalid component type ActionRow"));
            },
            ComponentType::Unknown(i) => {
                return Err(DeError::custom(format_args!("Unknown component type {i}")));
            },
            other => {
                return Err(DeError::custom(format_args!(
                    "Component type {other:?} cannot be directly placed inside an ActionRow"
                )));
            },
        }
        .map_err(DeError::custom)
    }
}

impl Serialize for ActionRowComponent {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        match self {
            Self::Button(c) => c.serialize(serializer),
            Self::InputText(c) => c.serialize(serializer),
            Self::SelectMenu(c) => c.serialize(serializer),
        }
    }
}

impl From<Button> for ActionRowComponent {
    fn from(component: Button) -> Self {
        ActionRowComponent::Button(component)
    }
}

impl From<SelectMenu> for ActionRowComponent {
    fn from(component: SelectMenu) -> Self {
        ActionRowComponent::SelectMenu(component)
    }
}

#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ButtonKind {
    Link { url: String },
    Premium { sku_id: SkuId },
    NonLink { custom_id: String, style: ButtonStyle },
}

impl Serialize for ButtonKind {
    fn serialize<S>(&self, serializer: S) -> StdResult<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a> {
            style: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            url: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            custom_id: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            sku_id: Option<SkuId>,
        }

        let helper = match self {
            ButtonKind::Link {
                url,
            } => Helper {
                style: 5,
                url: Some(url),
                custom_id: None,
                sku_id: None,
            },
            ButtonKind::Premium {
                sku_id,
            } => Helper {
                style: 6,
                url: None,
                custom_id: None,
                sku_id: Some(*sku_id),
            },
            ButtonKind::NonLink {
                custom_id,
                style,
            } => Helper {
                style: (*style).into(),
                url: None,
                custom_id: Some(custom_id),
                sku_id: None,
            },
        };
        helper.serialize(serializer)
    }
}

/// A button component.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#button-button-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Button {
    /// The component type, it will always be [`ComponentType::Button`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The button kind and style.
    #[serde(flatten)]
    pub data: ButtonKind,
    /// The text which appears on the button.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The emoji of this button, if there is one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emoji: Option<ReactionType>,
    /// Whether the button is disabled.
    #[serde(default)]
    pub disabled: bool,
}

enum_number! {
    /// The style of a button.
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum ButtonStyle {
        Primary = 1,
        Secondary = 2,
        Success = 3,
        Danger = 4,
        // No Link, because we represent Link using enum variants
        _ => Unknown(u8),
    }
}

/// A select menu component.
///
/// Discord docs: [String Select](https://docs.discord.com/developers/components/reference#string-select), [User Select](https://docs.discord.com/developers/components/reference#user-select), [Role Select](https://docs.discord.com/developers/components/reference#role-select), [Mentionable Select](https://docs.discord.com/developers/components/reference#mentionable-select), [Channel Select](https://docs.discord.com/developers/components/reference#channel-select).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SelectMenu {
    /// The component type, which may either be [`ComponentType::StringSelect`],
    /// [`ComponentType::UserSelect`], [`ComponentType::RoleSelect`],
    /// [`ComponentType::MentionableSelect`], or [`ComponentType::ChannelSelect`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// An identifier defined by the developer for the select menu.
    pub custom_id: Option<String>,
    /// The options of this select menu.
    ///
    /// Required for [`ComponentType::StringSelect`] and unavailable for all others.
    #[serde(default)]
    pub options: Vec<SelectMenuOption>,
    /// List of channel types to include in the [`ComponentType::ChannelSelect`].
    #[serde(default)]
    pub channel_types: Vec<ChannelType>,
    /// The placeholder shown when nothing is selected.
    pub placeholder: Option<String>,
    /// The minimum number of selections allowed.
    pub min_values: Option<u8>,
    /// The maximum number of selections allowed.
    pub max_values: Option<u8>,
    /// Whether select menu is disabled.
    #[serde(default)]
    pub disabled: bool,
}

/// A select menu component options.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#string-select-select-option-structure)
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SelectMenuOption {
    /// The text displayed on this option.
    pub label: String,
    /// The value to be sent for this option.
    pub value: String,
    /// The description shown for this option.
    pub description: Option<String>,
    /// The emoji displayed on this option.
    pub emoji: Option<ReactionType>,
    /// Render this option as the default selection.
    #[serde(default)]
    pub default: bool,
}

/// An input text component for modal interactions
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-structure).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct InputText {
    /// The component type, it will always be [`ComponentType::InputText`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Developer-defined identifier for the input; max 100 characters
    pub custom_id: String,
    /// The [`InputTextStyle`]. Required when sending modal data.
    ///
    /// Discord docs are wrong here; it says the field is always sent in modal submit interactions
    /// but it's not. It's only required when _sending_ modal data to Discord.
    /// <https://github.com/discord/discord-api-docs/issues/6141>
    pub style: Option<InputTextStyle>,
    /// Label for this component; max 45 characters. Required when sending modal data.
    ///
    /// Discord docs are wrong here; it says the field is always sent in modal submit interactions
    /// but it's not. It's only required when _sending_ modal data to Discord.
    /// <https://github.com/discord/discord-api-docs/issues/6141>
    pub label: Option<String>,
    /// Minimum input length for a text input; min 0, max 4000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u16>,
    /// Maximum input length for a text input; min 1, max 4000
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u16>,
    /// Whether this component is required to be filled (defaults to true)
    #[serde(default = "default_true")]
    pub required: bool,
    /// When sending: Pre-filled value for this component; max 4000 characters (may be None).
    ///
    /// When receiving: The input from the user (always Some)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Custom placeholder text if the input is empty; max 100 characters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
}

enum_number! {
    /// The style of the input text
    ///
    /// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-styles).
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum InputTextStyle {
        Short = 1,
        Paragraph = 2,
        _ => Unknown(u8),
    }
}

/// Media object for Discord Components V2 (thumbnails, media galleries, files).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct UnfurlMedia {
    /// The URL of the media source.
    pub url: String,
    /// A proxied URL of media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_url: Option<String>,
    /// Height of media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Width of media.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Placeholder image data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    /// Placeholder version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder_version: Option<u32>,
}

impl UnfurlMedia {
    /// Constructs a new [`UnfurlMedia`] with the given source URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            proxy_url: None,
            height: None,
            width: None,
            placeholder: None,
            placeholder_version: None,
        }
    }
}

/// A markdown text display component in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-display).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct TextDisplay {
    /// Component type, always [`ComponentType::TextDisplay`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Text content supporting markdown formatting.
    pub content: String,
}

impl TextDisplay {
    /// Constructs a new [`TextDisplay`] with the provided content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            kind: ComponentType::TextDisplay,
            content: content.into(),
        }
    }
}

/// A thumbnail component in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#thumbnail).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Thumbnail {
    /// Component type, always [`ComponentType::Thumbnail`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The media object for the thumbnail.
    pub media: UnfurlMedia,
    /// Alt text description for accessibility.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the thumbnail is hidden behind a spoiler.
    #[serde(default)]
    pub spoiler: bool,
}

impl Thumbnail {
    /// Constructs a new [`Thumbnail`] with the given media.
    pub fn new(media: UnfurlMedia) -> Self {
        Self {
            kind: ComponentType::Thumbnail,
            media,
            description: None,
            spoiler: false,
        }
    }
}

/// An accessory element inside a [`Section`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SectionAccessory {
    Button(Button),
    Thumbnail(Thumbnail),
}

impl Serialize for SectionAccessory {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        match self {
            Self::Button(c) => c.serialize(serializer),
            Self::Thumbnail(c) => c.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for SectionAccessory {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::Button => {
                from_value(value).map(SectionAccessory::Button).map_err(DeError::custom)
            },
            ComponentType::Thumbnail => {
                from_value(value).map(SectionAccessory::Thumbnail).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "Invalid accessory component type {other:?} in section"
            ))),
        }
    }
}

impl From<Button> for SectionAccessory {
    fn from(b: Button) -> Self {
        Self::Button(b)
    }
}

impl From<Thumbnail> for SectionAccessory {
    fn from(t: Thumbnail) -> Self {
        Self::Thumbnail(t)
    }
}

/// A section component in Components V2, grouping text with an optional accessory.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#section).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct Section {
    /// Component type, always [`ComponentType::Section`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Sub-components inside this section (typically [`TextDisplay`]).
    #[serde(default)]
    pub components: Vec<TextDisplay>,
    /// Accessory component attached to this section (button or thumbnail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accessory: Option<SectionAccessory>,
}

impl Section {
    /// Creates a new empty [`Section`].
    pub fn new() -> Self {
        Self {
            kind: ComponentType::Section,
            components: Vec::new(),
            accessory: None,
        }
    }
}

impl Default for Section {
    fn default() -> Self {
        Self::new()
    }
}

/// An individual media item inside a [`MediaGallery`].
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct MediaGalleryItem {
    /// The media object.
    pub media: UnfurlMedia,
    /// Alt text description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether the media is behind a spoiler.
    #[serde(default)]
    pub spoiler: bool,
}

impl MediaGalleryItem {
    /// Constructs a new [`MediaGalleryItem`].
    pub fn new(media: UnfurlMedia) -> Self {
        Self {
            media,
            description: None,
            spoiler: false,
        }
    }
}

/// A media gallery component in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#media-gallery).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct MediaGallery {
    /// Component type, always [`ComponentType::MediaGallery`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// List of media gallery items.
    #[serde(default)]
    pub items: Vec<MediaGalleryItem>,
}

impl MediaGallery {
    /// Creates a new [`MediaGallery`] with items.
    pub fn new(items: Vec<MediaGalleryItem>) -> Self {
        Self {
            kind: ComponentType::MediaGallery,
            items,
        }
    }
}

/// A file attachment display component in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#file).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct FileComponent {
    /// Component type, always [`ComponentType::File`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The file media object.
    pub file: UnfurlMedia,
    /// Whether the file is behind a spoiler.
    #[serde(default)]
    pub spoiler: bool,
}

impl FileComponent {
    /// Creates a new [`FileComponent`].
    pub fn new(file: UnfurlMedia) -> Self {
        Self {
            kind: ComponentType::File,
            file,
            spoiler: false,
        }
    }
}

enum_number! {
    /// Spacing size for a [`Separator`].
    #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
    #[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
    #[serde(from = "u8", into = "u8")]
    #[non_exhaustive]
    pub enum SeparatorSpacing {
        Small = 1,
        Large = 2,
        _ => Unknown(u8),
    }
}

/// A visual separator line in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#separator).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub struct Separator {
    /// Component type, always [`ComponentType::Separator`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Whether to render a visual divider line (default true).
    #[serde(default = "default_true")]
    pub divider: bool,
    /// Spacing around the separator.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spacing: Option<SeparatorSpacing>,
}

impl Separator {
    /// Creates a new [`Separator`] with default divider.
    pub fn new() -> Self {
        Self {
            kind: ComponentType::Separator,
            divider: true,
            spacing: None,
        }
    }
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

/// A container grouping components with an accent color and optional spoiler.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#container).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Container {
    /// Component type, always [`ComponentType::Container`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// Accent color as an RGB integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accent_color: Option<u32>,
    /// Whether the entire container is spoilered.
    #[serde(default)]
    pub spoiler: bool,
    /// The sub-components inside this container.
    #[serde(default)]
    pub components: Vec<Component>,
}

impl Container {
    /// Creates a new empty [`Container`].
    pub fn new() -> Self {
        Self {
            kind: ComponentType::Container,
            accent_color: None,
            spoiler: false,
            components: Vec::new(),
        }
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}

/// A label component for forms and modals in Components V2.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#label).
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct LabelComponent {
    /// Component type, always [`ComponentType::Label`].
    #[serde(rename = "type")]
    pub kind: ComponentType,
    /// The text label.
    pub label: String,
    /// Optional description text under the label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The component being labeled.
    pub component: Box<ActionRowComponent>,
}

impl LabelComponent {
    /// Creates a new [`LabelComponent`].
    pub fn new(label: impl Into<String>, component: impl Into<ActionRowComponent>) -> Self {
        Self {
            kind: ComponentType::Label,
            label: label.into(),
            description: None,
            component: Box::new(component.into()),
        }
    }
}

/// Top-level Discord component enum representing both V1 ActionRow and V2 Components.
#[cfg_attr(feature = "typesize", derive(typesize::derive::TypeSize))]
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum Component {
    ActionRow(ActionRow),
    Section(Section),
    TextDisplay(TextDisplay),
    Thumbnail(Thumbnail),
    MediaGallery(MediaGallery),
    File(FileComponent),
    Separator(Separator),
    Container(Container),
    Label(LabelComponent),
}

impl Serialize for Component {
    fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
        match self {
            Self::ActionRow(c) => c.serialize(serializer),
            Self::Section(c) => c.serialize(serializer),
            Self::TextDisplay(c) => c.serialize(serializer),
            Self::Thumbnail(c) => c.serialize(serializer),
            Self::MediaGallery(c) => c.serialize(serializer),
            Self::File(c) => c.serialize(serializer),
            Self::Separator(c) => c.serialize(serializer),
            Self::Container(c) => c.serialize(serializer),
            Self::Label(c) => c.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Component {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
        let map = JsonMap::deserialize(deserializer)?;
        let raw_kind = map.get("type").ok_or_else(|| DeError::missing_field("type"))?.clone();
        let value = Value::from(map);

        match deserialize_val(raw_kind)? {
            ComponentType::ActionRow => {
                from_value(value).map(Component::ActionRow).map_err(DeError::custom)
            },
            ComponentType::Section => {
                from_value(value).map(Component::Section).map_err(DeError::custom)
            },
            ComponentType::TextDisplay => {
                from_value(value).map(Component::TextDisplay).map_err(DeError::custom)
            },
            ComponentType::Thumbnail => {
                from_value(value).map(Component::Thumbnail).map_err(DeError::custom)
            },
            ComponentType::MediaGallery => {
                from_value(value).map(Component::MediaGallery).map_err(DeError::custom)
            },
            ComponentType::File => from_value(value).map(Component::File).map_err(DeError::custom),
            ComponentType::Separator => {
                from_value(value).map(Component::Separator).map_err(DeError::custom)
            },
            ComponentType::Container => {
                from_value(value).map(Component::Container).map_err(DeError::custom)
            },
            ComponentType::Label => {
                from_value(value).map(Component::Label).map_err(DeError::custom)
            },
            other => Err(DeError::custom(format_args!(
                "Unknown or unsupported top-level component type {other:?}"
            ))),
        }
    }
}

impl From<ActionRow> for Component {
    fn from(r: ActionRow) -> Self {
        Self::ActionRow(r)
    }
}

impl From<Section> for Component {
    fn from(s: Section) -> Self {
        Self::Section(s)
    }
}

impl From<TextDisplay> for Component {
    fn from(t: TextDisplay) -> Self {
        Self::TextDisplay(t)
    }
}

impl From<Thumbnail> for Component {
    fn from(t: Thumbnail) -> Self {
        Self::Thumbnail(t)
    }
}

impl From<MediaGallery> for Component {
    fn from(m: MediaGallery) -> Self {
        Self::MediaGallery(m)
    }
}

impl From<FileComponent> for Component {
    fn from(f: FileComponent) -> Self {
        Self::File(f)
    }
}

impl From<Separator> for Component {
    fn from(s: Separator) -> Self {
        Self::Separator(s)
    }
}

impl From<Container> for Component {
    fn from(c: Container) -> Self {
        Self::Container(c)
    }
}

impl From<LabelComponent> for Component {
    fn from(l: LabelComponent) -> Self {
        Self::Label(l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{assert_json, json};

    #[test]
    fn test_button_serde() {
        let mut button = Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                custom_id: "hello".into(),
                style: ButtonStyle::Danger,
            },
            label: Some("a".into()),
            emoji: None,
            disabled: false,
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 4, "custom_id": "hello", "label": "a", "disabled": false}),
        );

        button.data = ButtonKind::Link {
            url: "https://google.com".into(),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 5, "url": "https://google.com", "label": "a", "disabled": false}),
        );

        button.data = ButtonKind::Premium {
            sku_id: 1234965026943668316.into(),
        };
        assert_json(
            &button,
            json!({"type": 2, "style": 6, "sku_id": "1234965026943668316", "label": "a", "disabled": false}),
        );
    }

    #[test]
    fn test_components_v2_serde() {
        let text_disp = TextDisplay::new("Hello Discord V2!");
        assert_json(&text_disp, json!({"type": 10, "content": "Hello Discord V2!"}));

        let sep = Separator {
            kind: ComponentType::Separator,
            divider: true,
            spacing: Some(SeparatorSpacing::Small),
        };
        assert_json(&sep, json!({"type": 14, "divider": true, "spacing": 1}));

        let media = UnfurlMedia::new("https://example.com/image.png");
        let thumb = Thumbnail {
            kind: ComponentType::Thumbnail,
            media,
            description: Some("Example thumbnail".into()),
            spoiler: false,
        };
        assert_json(
            &thumb,
            json!({
                "type": 11,
                "media": {"url": "https://example.com/image.png"},
                "description": "Example thumbnail",
                "spoiler": false
            }),
        );

        let container = Container {
            kind: ComponentType::Container,
            accent_color: Some(0xFF00FF),
            spoiler: false,
            components: vec![Component::TextDisplay(text_disp.clone())],
        };
        let serialized = serde_json::to_value(&container).unwrap();
        assert_eq!(serialized["type"], 15);
        assert_eq!(serialized["accent_color"], 0xFF00FF);
        assert_eq!(serialized["components"][0]["type"], 10);
        assert_eq!(serialized["components"][0]["content"], "Hello Discord V2!");

        let deserialized: Container = serde_json::from_value(serialized).unwrap();
        assert_eq!(deserialized.components.len(), 1);
        if let Component::TextDisplay(t) = &deserialized.components[0] {
            assert_eq!(t.content, "Hello Discord V2!");
        } else {
            panic!("Expected TextDisplay component");
        }
    }
}
