use oxcord_model::prelude::*;
use serde::Serialize;

use crate::internal::prelude::*;

/// A builder for creating a components action row in a message.
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#action-row).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateActionRow {
    Buttons(Vec<CreateButton>),
    SelectMenu(CreateSelectMenu),
    /// Only valid in modals!
    InputText(CreateInputText),
}

impl serde::Serialize for CreateActionRow {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &1_u8)?;

        match self {
            CreateActionRow::Buttons(buttons) => map.serialize_entry("components", &buttons)?,
            CreateActionRow::SelectMenu(select) => map.serialize_entry("components", &[select])?,
            CreateActionRow::InputText(input) => map.serialize_entry("components", &[input])?,
        }

        map.end()
    }
}

/// A builder for creating a button component in a message
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateButton(Button);

impl CreateButton {
    /// Creates a link button to the given URL. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
    pub fn new_link(url: impl Into<String>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::Link {
                url: url.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Creates a new premium button associated with the given SKU.
    ///
    /// Clicking this button _will not_ trigger an interaction event in your bot.
    pub fn new_premium(sku_id: impl Into<SkuId>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::Premium {
                sku_id: sku_id.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Creates a normal button with the given custom ID. You must also set [`Self::label`] and/or
    /// [`Self::emoji`] after this.
    pub fn new(custom_id: impl Into<String>) -> Self {
        Self(Button {
            kind: ComponentType::Button,
            data: ButtonKind::NonLink {
                style: ButtonStyle::Primary,
                custom_id: custom_id.into(),
            },
            label: None,
            emoji: None,
            disabled: false,
        })
    }

    /// Sets the custom id of the button, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    ///
    /// Has no effect on link buttons and premium buttons.
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        if let ButtonKind::NonLink {
            custom_id, ..
        } = &mut self.0.data
        {
            *custom_id = id.into();
        }
        self
    }

    /// Sets the style of this button.
    ///
    /// Has no effect on link buttons and premium buttons.
    pub fn style(mut self, new_style: ButtonStyle) -> Self {
        if let ButtonKind::NonLink {
            style, ..
        } = &mut self.0.data
        {
            *style = new_style;
        }
        self
    }

    /// Sets label of the button.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    /// Sets emoji of the button.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.0.emoji = Some(emoji.into());
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.0.disabled = disabled;
        self
    }
}

impl From<Button> for CreateButton {
    fn from(button: Button) -> Self {
        Self(button)
    }
}

struct CreateSelectMenuDefault(Mention);

impl Serialize for CreateSelectMenuDefault {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let (id, kind) = match self.0 {
            Mention::Channel(c) => (c.get(), "channel"),
            Mention::Role(r) => (r.get(), "role"),
            Mention::User(u) => (u.get(), "user"),
        };

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("id", &id)?;
        map.serialize_entry("type", kind)?;
        map.end()
    }
}

/// Discord docs: [String Select](https://docs.discord.com/developers/components/reference#string-select), [User Select](https://docs.discord.com/developers/components/reference#user-select), [Role Select](https://docs.discord.com/developers/components/reference#role-select), [Mentionable Select](https://docs.discord.com/developers/components/reference#mentionable-select), [Channel Select](https://docs.discord.com/developers/components/reference#channel-select).
#[derive(Clone, Debug, PartialEq)]
pub enum CreateSelectMenuKind {
    String { options: Vec<CreateSelectMenuOption> },
    User { default_users: Option<Vec<UserId>> },
    Role { default_roles: Option<Vec<RoleId>> },
    Mentionable { default_users: Option<Vec<UserId>>, default_roles: Option<Vec<RoleId>> },
    Channel { channel_types: Option<Vec<ChannelType>>, default_channels: Option<Vec<ChannelId>> },
}

impl Serialize for CreateSelectMenuKind {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Json<'a> {
            #[serde(rename = "type")]
            kind: u8,
            #[serde(skip_serializing_if = "Option::is_none")]
            options: Option<&'a [CreateSelectMenuOption]>,
            #[serde(skip_serializing_if = "Option::is_none")]
            channel_types: Option<&'a [ChannelType]>,
            #[serde(skip_serializing_if = "Vec::is_empty")]
            default_values: Vec<CreateSelectMenuDefault>,
        }

        #[allow(clippy::ref_option)]
        fn map<I: Into<Mention> + Copy>(
            values: &Option<Vec<I>>,
        ) -> impl Iterator<Item = CreateSelectMenuDefault> + '_ {
            // Calling `.iter().flatten()` on the `Option` treats `None` like an empty vec
            values.iter().flatten().map(|&i| CreateSelectMenuDefault(i.into()))
        }

        #[rustfmt::skip]
        let default_values = match self {
            Self::String { .. } => vec![],
            Self::User { default_users: default_values } => map(default_values).collect(),
            Self::Role { default_roles: default_values } => map(default_values).collect(),
            Self::Mentionable { default_users, default_roles } => {
                let users = map(default_users);
                let roles = map(default_roles);
                users.chain(roles).collect()
            },
            Self::Channel { channel_types: _, default_channels: default_values } => map(default_values).collect(),
        };

        #[rustfmt::skip]
        let json = Json {
            kind: match self {
                Self::String { .. } => 3,
                Self::User { .. } => 5,
                Self::Role { .. } => 6,
                Self::Mentionable { .. } => 7,
                Self::Channel { .. } => 8,
            },
            options: match self {
                Self::String { options } => Some(options),
                _ => None,
            },
            channel_types: match self {
                Self::Channel { channel_types, default_channels: _ } => channel_types.as_deref(),
                _ => None,
            },
            default_values,
        };

        json.serialize(serializer)
    }
}

/// A builder for creating a select menu component in a message
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#component-object).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateSelectMenu {
    custom_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_values: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disabled: Option<bool>,

    #[serde(flatten)]
    kind: CreateSelectMenuKind,
}

impl CreateSelectMenu {
    /// Creates a builder with given custom id (a developer-defined identifier), and a list of
    /// options, leaving all other fields empty.
    pub fn new(custom_id: impl Into<String>, kind: CreateSelectMenuKind) -> Self {
        Self {
            custom_id: custom_id.into(),
            placeholder: None,
            min_values: None,
            max_values: None,
            disabled: None,
            kind,
        }
    }

    /// The placeholder of the select menu.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.placeholder = Some(label.into());
        self
    }

    /// Sets the custom id of the select menu, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.custom_id = id.into();
        self
    }

    /// Sets the minimum values for the user to select.
    pub fn min_values(mut self, min: u8) -> Self {
        self.min_values = Some(min);
        self
    }

    /// Sets the maximum values for the user to select.
    pub fn max_values(mut self, max: u8) -> Self {
        self.max_values = Some(max);
        self
    }

    /// Sets the disabled state for the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

/// A builder for creating an option of a select menu component in a message
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#string-select-select-option-structure)
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateSelectMenuOption {
    label: String,
    value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    emoji: Option<ReactionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default: Option<bool>,
}

impl CreateSelectMenuOption {
    /// Creates a select menu option with the given label and value, leaving all other fields
    /// empty.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            description: None,
            emoji: None,
            default: None,
        }
    }

    /// Sets the label of this option, replacing the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the value of this option, replacing the current value as set in [`Self::new`].
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Sets the description shown on this option.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets emoji of the option.
    pub fn emoji(mut self, emoji: impl Into<ReactionType>) -> Self {
        self.emoji = Some(emoji.into());
        self
    }

    /// Sets this option as selected by default.
    pub fn default_selection(mut self, default: bool) -> Self {
        self.default = Some(default);
        self
    }
}

/// A builder for creating an input text component in a modal
///
/// [Discord docs](https://docs.discord.com/developers/components/reference#text-input-text-input-structure).
#[derive(Clone, Debug, Serialize, PartialEq)]
#[must_use]
pub struct CreateInputText(InputText);

impl CreateInputText {
    /// Creates a text input with the given style, label, and custom id (a developer-defined
    /// identifier), leaving all other fields empty.
    pub fn new(
        style: InputTextStyle,
        label: impl Into<String>,
        custom_id: impl Into<String>,
    ) -> Self {
        Self(InputText {
            style: Some(style),
            label: Some(label.into()),
            custom_id: custom_id.into(),

            placeholder: None,
            min_length: None,
            max_length: None,
            value: None,
            required: true,

            kind: ComponentType::InputText,
        })
    }

    /// Sets the style of this input text. Replaces the current value as set in [`Self::new`].
    pub fn style(mut self, kind: InputTextStyle) -> Self {
        self.0.style = Some(kind);
        self
    }

    /// Sets the label of this input text. Replaces the current value as set in [`Self::new`].
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.0.label = Some(label.into());
        self
    }

    /// Sets the custom id of the input text, a developer-defined identifier. Replaces the current
    /// value as set in [`Self::new`].
    pub fn custom_id(mut self, id: impl Into<String>) -> Self {
        self.0.custom_id = id.into();
        self
    }

    /// Sets the placeholder of this input text.
    pub fn placeholder(mut self, label: impl Into<String>) -> Self {
        self.0.placeholder = Some(label.into());
        self
    }

    /// Sets the minimum length required for the input text
    pub fn min_length(mut self, min: u16) -> Self {
        self.0.min_length = Some(min);
        self
    }

    /// Sets the maximum length required for the input text
    pub fn max_length(mut self, max: u16) -> Self {
        self.0.max_length = Some(max);
        self
    }

    /// Sets the value of this input text.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.0.value = Some(value.into());
        self
    }

    /// Sets if the input text is required
    pub fn required(mut self, required: bool) -> Self {
        self.0.required = required;
        self
    }
}

/// A builder for creating a [`TextDisplay`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateTextDisplay(TextDisplay);

impl CreateTextDisplay {
    /// Constructs a new [`CreateTextDisplay`] with the given markdown content.
    pub fn new(content: impl Into<String>) -> Self {
        Self(TextDisplay::new(content))
    }

    /// Sets the text content.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.0.content = content.into();
        self
    }
}

impl Serialize for CreateTextDisplay {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// A builder for creating a [`Thumbnail`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateThumbnail(Thumbnail);

impl CreateThumbnail {
    /// Constructs a new [`CreateThumbnail`] with the given image URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self(Thumbnail::new(UnfurlMedia::new(url)))
    }

    /// Sets the alt text description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.0.description = Some(desc.into());
        self
    }

    /// Sets whether the thumbnail is hidden behind a spoiler.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.0.spoiler = spoiler;
        self
    }
}

impl Serialize for CreateThumbnail {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// An accessory element inside a [`CreateSection`].
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateSectionAccessory {
    Button(CreateButton),
    Thumbnail(CreateThumbnail),
}

impl Serialize for CreateSectionAccessory {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Button(b) => b.serialize(serializer),
            Self::Thumbnail(t) => t.serialize(serializer),
        }
    }
}

impl From<CreateButton> for CreateSectionAccessory {
    fn from(b: CreateButton) -> Self {
        Self::Button(b)
    }
}

impl From<CreateThumbnail> for CreateSectionAccessory {
    fn from(t: CreateThumbnail) -> Self {
        Self::Thumbnail(t)
    }
}

/// A builder for creating a [`Section`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateSection {
    components: Vec<CreateTextDisplay>,
    accessory: Option<CreateSectionAccessory>,
}

impl CreateSection {
    /// Creates a new empty [`CreateSection`].
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            accessory: None,
        }
    }

    /// Adds markdown text to the section.
    pub fn add_text(mut self, text: impl Into<String>) -> Self {
        self.components.push(CreateTextDisplay::new(text));
        self
    }

    /// Adds a [`CreateTextDisplay`] to the section.
    pub fn add_text_display(mut self, text_display: CreateTextDisplay) -> Self {
        self.components.push(text_display);
        self
    }

    /// Sets an accessory component (button or thumbnail).
    pub fn accessory(mut self, accessory: impl Into<CreateSectionAccessory>) -> Self {
        self.accessory = Some(accessory.into());
        self
    }
}

impl Default for CreateSection {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for CreateSection {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &9_u8)?;
        map.serialize_entry("components", &self.components)?;
        if let Some(acc) = &self.accessory {
            map.serialize_entry("accessory", acc)?;
        }
        map.end()
    }
}

/// A builder for creating a [`MediaGalleryItem`].
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateMediaGalleryItem(MediaGalleryItem);

impl CreateMediaGalleryItem {
    /// Constructs a new [`CreateMediaGalleryItem`].
    pub fn new(url: impl Into<String>) -> Self {
        Self(MediaGalleryItem::new(UnfurlMedia::new(url)))
    }

    /// Sets the description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.0.description = Some(desc.into());
        self
    }

    /// Sets whether the item is spoilered.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.0.spoiler = spoiler;
        self
    }
}

impl Serialize for CreateMediaGalleryItem {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// A builder for creating a [`MediaGallery`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateMediaGallery {
    items: Vec<CreateMediaGalleryItem>,
}

impl CreateMediaGallery {
    /// Creates a new empty [`CreateMediaGallery`].
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }

    /// Adds an item to the media gallery.
    pub fn add_item(mut self, item: CreateMediaGalleryItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets all items in the media gallery.
    pub fn items(mut self, items: Vec<CreateMediaGalleryItem>) -> Self {
        self.items = items;
        self
    }
}

impl Default for CreateMediaGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for CreateMediaGallery {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("type", &12_u8)?;
        map.serialize_entry("items", &self.items)?;
        map.end()
    }
}

/// A builder for creating a [`FileComponent`] (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateFileComponent(FileComponent);

impl CreateFileComponent {
    /// Constructs a new [`CreateFileComponent`] with the given file URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self(FileComponent::new(UnfurlMedia::new(url)))
    }

    /// Sets whether the file is behind a spoiler.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.0.spoiler = spoiler;
        self
    }
}

impl Serialize for CreateFileComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// A builder for creating a [`Separator`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateSeparator(Separator);

impl CreateSeparator {
    /// Constructs a new [`CreateSeparator`].
    pub fn new() -> Self {
        Self(Separator::new())
    }

    /// Sets whether a divider line is rendered (default true).
    pub fn divider(mut self, divider: bool) -> Self {
        self.0.divider = divider;
        self
    }

    /// Sets the spacing size around the separator.
    pub fn spacing(mut self, spacing: SeparatorSpacing) -> Self {
        self.0.spacing = Some(spacing);
        self
    }
}

impl Default for CreateSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for CreateSeparator {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

/// A builder for creating a [`Container`] component (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateContainer {
    accent_color: Option<u32>,
    spoiler: bool,
    components: Vec<CreateComponent>,
}

impl CreateContainer {
    /// Constructs a new empty [`CreateContainer`].
    pub fn new() -> Self {
        Self {
            accent_color: None,
            spoiler: false,
            components: Vec::new(),
        }
    }

    /// Sets an accent color as an RGB integer.
    pub fn accent_color(mut self, color: u32) -> Self {
        self.accent_color = Some(color);
        self
    }

    /// Sets whether the entire container is spoilered.
    pub fn spoiler(mut self, spoiler: bool) -> Self {
        self.spoiler = spoiler;
        self
    }

    /// Adds a child component inside this container.
    pub fn add_component(mut self, component: impl Into<CreateComponent>) -> Self {
        self.components.push(component.into());
        self
    }

    /// Sets all child components inside this container.
    pub fn components(mut self, components: Vec<CreateComponent>) -> Self {
        self.components = components;
        self
    }
}

impl Default for CreateContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serialize for CreateContainer {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &15_u8)?;
        if let Some(color) = self.accent_color {
            map.serialize_entry("accent_color", &color)?;
        }
        if self.spoiler {
            map.serialize_entry("spoiler", &true)?;
        }
        map.serialize_entry("components", &self.components)?;
        map.end()
    }
}

/// A builder for creating a [`LabelComponent`] for modals and forms (Components V2).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub struct CreateLabel {
    label: String,
    description: Option<String>,
    component: Box<CreateActionRowComponent>,
}

impl CreateLabel {
    /// Constructs a new [`CreateLabel`].
    pub fn new(label: impl Into<String>, component: impl Into<CreateActionRowComponent>) -> Self {
        Self {
            label: label.into(),
            description: None,
            component: Box::new(component.into()),
        }
    }

    /// Sets description text under the label.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl Serialize for CreateLabel {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap as _;

        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("type", &16_u8)?;
        map.serialize_entry("label", &self.label)?;
        if let Some(desc) = &self.description {
            map.serialize_entry("description", desc)?;
        }
        map.serialize_entry("component", &self.component)?;
        map.end()
    }
}

/// Component which can be placed inside an ActionRow or Label.
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateActionRowComponent {
    Button(CreateButton),
    SelectMenu(CreateSelectMenu),
    InputText(CreateInputText),
}

impl Serialize for CreateActionRowComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Button(b) => b.serialize(serializer),
            Self::SelectMenu(s) => s.serialize(serializer),
            Self::InputText(i) => i.serialize(serializer),
        }
    }
}

impl From<CreateButton> for CreateActionRowComponent {
    fn from(b: CreateButton) -> Self {
        Self::Button(b)
    }
}

impl From<CreateSelectMenu> for CreateActionRowComponent {
    fn from(s: CreateSelectMenu) -> Self {
        Self::SelectMenu(s)
    }
}

impl From<CreateInputText> for CreateActionRowComponent {
    fn from(i: CreateInputText) -> Self {
        Self::InputText(i)
    }
}

/// A top-level component builder that can represent either a V1 [`CreateActionRow`]
/// or any Components V2 component ([`CreateContainer`], [`CreateSection`], [`CreateTextDisplay`],
/// etc.).
#[derive(Clone, Debug, PartialEq)]
#[must_use]
pub enum CreateComponent {
    ActionRow(CreateActionRow),
    Section(CreateSection),
    TextDisplay(CreateTextDisplay),
    Thumbnail(CreateThumbnail),
    MediaGallery(CreateMediaGallery),
    File(CreateFileComponent),
    Separator(CreateSeparator),
    Container(CreateContainer),
    Label(CreateLabel),
}

impl Serialize for CreateComponent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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

impl From<CreateActionRow> for CreateComponent {
    fn from(r: CreateActionRow) -> Self {
        Self::ActionRow(r)
    }
}

impl From<CreateSection> for CreateComponent {
    fn from(s: CreateSection) -> Self {
        Self::Section(s)
    }
}

impl From<CreateTextDisplay> for CreateComponent {
    fn from(t: CreateTextDisplay) -> Self {
        Self::TextDisplay(t)
    }
}

impl From<CreateThumbnail> for CreateComponent {
    fn from(t: CreateThumbnail) -> Self {
        Self::Thumbnail(t)
    }
}

impl From<CreateMediaGallery> for CreateComponent {
    fn from(m: CreateMediaGallery) -> Self {
        Self::MediaGallery(m)
    }
}

impl From<CreateFileComponent> for CreateComponent {
    fn from(f: CreateFileComponent) -> Self {
        Self::File(f)
    }
}

impl From<CreateSeparator> for CreateComponent {
    fn from(s: CreateSeparator) -> Self {
        Self::Separator(s)
    }
}

impl From<CreateContainer> for CreateComponent {
    fn from(c: CreateContainer) -> Self {
        Self::Container(c)
    }
}

impl From<CreateLabel> for CreateComponent {
    fn from(l: CreateLabel) -> Self {
        Self::Label(l)
    }
}

#[cfg(test)]
mod tests {
    use oxcord_model::channel::MessageFlags;

    use super::*;
    use crate::CreateMessage;

    #[test]
    fn test_create_components_v2_serialization() {
        let text = CreateTextDisplay::new("## Announcement\nWelcome to OXCORD Components V2!");
        let sep = CreateSeparator::new().divider(true).spacing(SeparatorSpacing::Small);
        let button = CreateButton::new("btn_click").label("Click Me!");
        let section = CreateSection::new()
            .add_text("Click this accessory button on the right:")
            .accessory(button);

        let container = CreateContainer::new()
            .accent_color(0x5865F2)
            .add_component(text)
            .add_component(sep)
            .add_component(section);

        let json = serde_json::to_value(&container).unwrap();
        assert_eq!(json["type"], 15);
        assert_eq!(json["accent_color"], 0x5865F2);
        assert_eq!(json["components"][0]["type"], 10);
        assert_eq!(json["components"][1]["type"], 14);
        assert_eq!(json["components"][2]["type"], 9);
        assert_eq!(json["components"][2]["accessory"]["type"], 2);

        // Test that CreateMessage automatically sets IS_COMPONENTS_V2 flag
        let msg = CreateMessage::new().components_v2(vec![container.into()]);
        let msg_json = serde_json::to_value(&msg).unwrap();
        assert_eq!(
            msg_json["flags"].as_u64().unwrap() & MessageFlags::IS_COMPONENTS_V2.bits(),
            MessageFlags::IS_COMPONENTS_V2.bits()
        );
        assert_eq!(msg_json["components"][0]["type"], 15);
    }
}
