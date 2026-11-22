# oxcord-framework

A high-level, poise-style command framework for [Oxcord](https://github.com/wumx-labs/oxcord).

Write one async function and get both **slash commands** and **prefix commands** for free.

## Features

- **Unified commands** — `#[command(slash_command, prefix_command)]` handles both kinds
- **Argument extraction** — typed option parsing from slash or prefix invocations
- **Cooldowns** — per-User / per-Guild / per-Channel / per-Member / Global scopes
- **Autocomplete** — attach async autocomplete handlers to any named option
- **Permission checks** — `guild_only`, `owner_only`, `required_permissions`
- **Subcommands** — group commands with `.subcommand()`
- **Ergonomic errors** — structured `FrameworkError` with `thiserror`

## Quick start

```toml
[dependencies]
oxcord-framework = { version = "0.0.1", path = "../../crates/oxcord-framework" }
oxcord-macros    = { version = "0.0.1", path = "../../crates/oxcord-macros" }
```

```rust
use oxcord_framework::{Framework, FrameworkOptions, Context};

#[oxcord_macros::command(slash_command, prefix_command, description = "Ping the bot")]
async fn ping(ctx: Context<'_>) -> oxcord_framework::Result<()> {
    ctx.say("Pong! 🏓").await
}

#[tokio::main]
async fn main() {
    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![ping()],
            prefix: Some("!".into()),
            auto_register_slash: true,
            ..Default::default()
        })
        .build();

    println!("Registered {} command(s)", framework.commands().len());
}
```

## Cooldown example

```rust
use oxcord_framework::cooldown::{CooldownConfig, CooldownManager, CooldownScope};
use std::time::Duration;

let cooldown = CooldownManager::new(CooldownConfig::new(
    CooldownScope::Member,
    Duration::from_secs(30),
));
```

## License

Apache-2.0 OR ISC — see [LICENSE](../../LICENSE) and [NOTICE](../../NOTICE).
