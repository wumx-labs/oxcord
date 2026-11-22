//! Demonstrates the oxcord-framework declarative command system.
//!
//! This example shows how to:
//! - Define slash + prefix commands with a single function
//! - Add prefix aliases
//! - Enforce guild-only restriction
//! - Set up a cooldown
//! - Use autocomplete on a slash option
//!
//! To run:
//! ```bash
//! cargo run -p e26_declarative_slash_bot
//! ```

use std::time::Duration;

use oxcord_framework::cooldown::{CooldownConfig, CooldownManager, CooldownScope};
use oxcord_framework::{Context, Framework, FrameworkOptions};

// --- Commands ---

/// A simple ping command available as both /ping and !ping.
#[oxcord_macros::command(
    slash_command,
    prefix_command,
    description = "Check if the bot is alive",
    aliases("p"),
    category = "General"
)]
async fn ping(ctx: Context<'_>) -> oxcord_framework::Result<()> {
    ctx.say("Pong! 🏓").await
}

/// A guild-only server info command with a 10-second per-user cooldown.
#[oxcord_macros::command(
    slash_command,
    prefix_command,
    description = "Show information about this server",
    aliases("server", "si"),
    guild_only,
    category = "Utility"
)]
async fn server_info(ctx: Context<'_>) -> oxcord_framework::Result<()> {
    let guild = ctx.guild_id.map(|g| g.to_string()).unwrap_or_else(|| "DM".to_owned());
    ctx.say(format!("Guild ID: {guild}")).await
}

/// A restricted owner-only reload command.
#[oxcord_macros::command(
    slash_command,
    description = "Reload bot configuration (owner only)",
    owner_only,
    category = "Admin"
)]
async fn reload(ctx: Context<'_>) -> oxcord_framework::Result<()> {
    ctx.say("✅ Configuration reloaded.").await
}

// --- Main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Build the framework with all three commands registered.
    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![ping(), server_info(), reload()],
            prefix: Some("!".into()),
            auto_register_slash: true,
            ..Default::default()
        })
        .build();

    // Print registered commands.
    println!("Registered commands:");
    for cmd in framework.commands() {
        println!(
            "  {:>15}  kind={:?}  aliases={:?}  guild_only={:?}  owner_only={}",
            cmd.name, cmd.kind, cmd.aliases, cmd.guild_only, cmd.owner_only,
        );
    }

    // Example: set up a cooldown for `server_info`.
    // In a real bot you'd attach this to the Dispatcher.
    let _cooldown =
        CooldownManager::new(CooldownConfig::new(CooldownScope::User, Duration::from_secs(10)));

    println!("\nIn a full Oxcord integration, pass `framework` to `ClientBuilder::framework()`.");
}
