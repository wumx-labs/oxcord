# Slash Commands, Autocomplete & Modals

OXCORD provides first-class support for Discord's application command system, message components (Buttons, Select Menus, Components V2), and dynamic modals.

---

## ⚡ Handling Slash Commands

Slash commands are received via the `interaction_create` event on your `EventHandler`:

```rust
use oxcord::async_trait;
use oxcord::builder::*;
use oxcord::model::application::Interaction;
use oxcord::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::Command(command) => {
                let content = match command.data.name.as_str() {
                    "ping" => "Pong! 🏓".to_string(),
                    "hello" => format!("Hello <@{}>!", command.user.id),
                    _ => "Unknown command".to_string(),
                };

                let response = CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(content),
                );

                if let Err(why) = command.create_response(&ctx.http, response).await {
                    eprintln!("Cannot respond to slash command: {why:?}");
                }
            }
            Interaction::Autocomplete(autocomplete) => {
                // Dynamic autocomplete choices
                let choices = vec![
                    "Option Alpha".into(),
                    "Option Beta".into(),
                    "Option Gamma".into(),
                ];

                let response = CreateInteractionResponse::Autocomplete(
                    CreateAutocompleteResponse::default().set_choices(choices),
                );

                let _ = autocomplete.create_response(&ctx.http, response).await;
            }
            _ => {}
        }
    }
}
```

---

## 🪟 Interactive Modals

You can prompt users with interactive text input modals:

```rust
use oxcord::builder::*;
use oxcord::model::application::CommandInteraction;

async fn prompt_modal(ctx: &Context, command: &CommandInteraction) {
    let text_input = CreateInputText::new(
        InputTextStyle::Paragraph,
        "Feedback",
        "feedback_input",
    )
    .placeholder("Enter your thoughts here...")
    .required(true);

    let modal = CreateModal::new("user_feedback_modal", "Submit Feedback")
        .components(vec![CreateActionRow::InputText(text_input)]);

    let response = CreateInteractionResponse::Modal(modal);
    command.create_response(&ctx.http, response).await.unwrap();
}
```

---

## 🔘 Components V2 (Interaction Type 9)

OXCORD natively supports Discord's modern **Components V2** (Premium / Deferred and nested interactive components):

```rust
use oxcord::builder::*;

let response = CreateInteractionResponse::Message(
    CreateInteractionResponseMessage::new()
        .content("Choose an action below:")
        .button(CreateButton::new("btn_primary").label("Confirm").style(ButtonStyle::Primary))
        .button(CreateButton::new("btn_danger").label("Cancel").style(ButtonStyle::Danger)),
);
```
