# Basic Ping Bot (e01)

The simplest possible OXCORD bot — responds `Pong!` to `~ping`.

> [!NOTE]
> Source code: [`examples/e01_basic_ping_bot/`](https://github.com/wumx-labs/oxcord/tree/main/examples/e01_basic_ping_bot)

## Running

```sh
DISCORD_TOKEN=your_token cargo run --example e01_basic_ping_bot
```

## Code

```rust
use oxcord::async_trait;
use oxcord::client::{Client, Context, EventHandler};
use oxcord::model::channel::Message;
use oxcord::model::gateway::{GatewayIntents, Ready};
use oxcord::ops::MessageExt;
use oxcord::prelude::TypeMapKey;
use std::env;

struct Handler;

#[oxcord::async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "~ping" {
            if let Err(e) = msg.reply(&ctx, "Pong!").await {
                eprintln!("Error sending message: {e}");
            }
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Failed to create client");

    client.start().await.expect("Client error");
}
```

## See Also

- [Getting Started](/guide/getting-started)
- [Architecture](/guide/architecture)
