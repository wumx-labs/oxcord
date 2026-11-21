# Zero-Alloc Webhooks (Serverless Bots)

Run Discord bots with **zero WebSocket connections**! Receive Slash Commands, Autocomplete queries, and Modal submissions directly over HTTP POST via Discord's Interactions Endpoint URL.

---

## ⚡ Zero-Alloc Ed25519 Verification

Discord requires all incoming webhook POST requests to be cryptographically verified using Ed25519.

OXCORD optimizes this with **zero heap allocations** for requests under 4 KiB using internal stack-allocated buffers:

```rust
use oxcord::interactions_endpoint::Verifier;

let verifier = Verifier::new("YOUR_DISCORD_APP_PUBLIC_KEY_HEX");

// Zero heap allocation check:
if verifier.verify_fast(signature, timestamp, body).is_err() {
    // Return HTTP 401 Unauthorized
}
```

---

## 🛠️ Unified Interaction Handler

Use [`InteractionHandler`] and [`process_interaction`] to handle incoming requests in any web framework (tiny_http, Axum, Actix-web, Tower, Warp):

```rust
use oxcord::async_trait;
use oxcord::builder::*;
use oxcord::interactions_endpoint::{
    process_interaction, InteractionHandler, Verifier,
};
use oxcord::model::application::*;

struct BotHandler;

#[async_trait]
impl InteractionHandler for BotHandler {
    // Ping/Pong is handled automatically (CreateInteractionResponse::Pong)

    async fn on_command(&self, cmd: &CommandInteraction) -> Option<CreateInteractionResponse> {
        Some(CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new().content(format!("Hello from Webhook! <@{}>", cmd.user.id)),
        ))
    }

    async fn on_autocomplete(&self, _: &CommandInteraction) -> Option<CreateInteractionResponse> {
        Some(CreateInteractionResponse::Autocomplete(
            CreateAutocompleteResponse::default().set_choices(vec!["Option 1".into()]),
        ))
    }
}

// In your HTTP route handler:
let output = process_interaction(&verifier, &BotHandler, signature, timestamp, body).await?;
// Returns status 200 with application/json, or 401 if forged!
```
