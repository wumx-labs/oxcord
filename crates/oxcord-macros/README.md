# oxcord-macros

Proc-macro helpers for [`oxcord-framework`](../oxcord-framework).

## `#[command]`

```rust
use oxcord_framework::Context;

#[oxcord_macros::command(
    slash_command,
    prefix_command,
    description = "Show server info",
    aliases("serverinfo", "si"),
    guild_only,
    category = "Utility"
)]
async fn server_info(ctx: Context<'_>) -> oxcord_framework::Result<()> {
    ctx.say("Server info coming soon!").await
}
```

### Supported attributes

| Attribute | Type | Description |
|---|---|---|
| `slash_command` | flag | Register as a slash command |
| `prefix_command` | flag | Register as a prefix command |
| `name = "..."` | string | Override command name (default: fn name) |
| `description = "..."` | string | One-line description for Discord |
| `aliases("a", "b")` | list | Prefix aliases |
| `guild_only` | flag | Restrict to guilds |
| `owner_only` | flag | Restrict to bot owners |
| `category = "..."` | string | Group in the help page |

## License

Apache-2.0 OR ISC — see [LICENSE](../../LICENSE) and [NOTICE](../../NOTICE).
