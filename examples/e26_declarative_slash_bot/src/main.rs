//! Demonstrates the oxcord-framework declarative command system, wired all
//! the way to a real gateway connection.
//!
//! This example shows how to:
//! - Define slash + prefix commands with a single function
//! - Accept a typed, optional argument (`greet`'s `name`)
//! - Add prefix aliases
//! - Enforce guild-only restriction
//! - Set up a cooldown
//! - Pass the framework straight to `ClientBuilder::framework()`
//!
//! To run:
//! ```bash
//! DISCORD_TOKEN=... cargo run -p e26_declarative_slash_bot
//! ```

use oxcord::client::{Client, EventHandler};
use oxcord::model::gateway::GatewayIntents;
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
    category = "Utility",
    cooldown_secs = 10,
    cooldown_scope = "user"
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

/// A typed-argument command: `name` is pulled out of the slash option (by
/// name) or the prefix text (positionally) automatically — no manual
/// `InvocationData` matching required. `Option<String>` makes it optional.
#[oxcord_macros::command(
    slash_command,
    prefix_command,
    description = "Greet someone by name",
    category = "General"
)]
async fn greet(ctx: Context<'_>, name: Option<String>) -> oxcord_framework::Result<()> {
    let name = name.unwrap_or_else(|| "there".to_owned());
    ctx.say(format!("Hello, {name}! 👋")).await
}

// --- A command with autocomplete, built by hand ---
//
// `#[oxcord_macros::command]` doesn't have an attribute for wiring an
// autocomplete handler yet, so this one is assembled directly with
// `Command::builder` to prove the dispatch path (an `InteractionCreate` of
// kind `Autocomplete`, routed through `Framework::dispatch_autocomplete` in
// `oxcord::framework::v2_bridge`, responding with suggestions) actually
// works end to end.

const FRUITS: &[&str] = &["apple", "apricot", "avocado", "banana", "blueberry", "cherry"];

async fn fruit(ctx: oxcord_framework::Context<'_>) -> oxcord_framework::Result<()> {
    let picked: Option<String> = ctx.option("name")?;
    ctx.say(format!("You picked: {}", picked.unwrap_or_else(|| "nothing".to_owned()))).await
}

async fn fruit_autocomplete(
    _ctx: oxcord_framework::Context<'_>,
    partial: String,
) -> oxcord_framework::Result<Vec<String>> {
    Ok(FRUITS.iter().filter(|f| f.starts_with(partial.as_str())).map(|f| (*f).to_owned()).take(25).collect())
}

fn fruit_command() -> oxcord_framework::Command {
    fn handler<'a>(
        ctx: oxcord_framework::Context<'a>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = oxcord_framework::Result<()>> + Send + 'a>> {
        Box::pin(fruit(ctx))
    }
    fn autocomplete<'a>(
        ctx: oxcord_framework::Context<'a>,
        partial: String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = oxcord_framework::Result<Vec<String>>> + Send + 'a>> {
        Box::pin(fruit_autocomplete(ctx, partial))
    }

    oxcord_framework::Command::builder("fruit")
        .description("Pick a fruit (with autocomplete)")
        .kind(oxcord_framework::CommandKind::Slash)
        .category("General")
        .option(oxcord_framework::CommandOptionSpec {
            name: "name".to_owned(),
            description: "Which fruit".to_owned(),
            kind: oxcord::model::application::CommandOptionType::String,
            required: true,
        })
        .autocomplete_for("name", autocomplete as oxcord_framework::AutocompleteFn)
        .handler(handler as oxcord_framework::CommandFn)
        .build()
}

// --- Main ---

/// The gateway event handler is still required by `Client::builder`, but
/// with a framework attached it can stay empty: `Framework::dispatch` (via
/// the bridge in `oxcord::framework::v2_bridge`) receives every event and
/// routes `Message`/`InteractionCreate` into the right command.
struct Handler;

impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Build the framework with all four commands registered.
    let framework = Framework::builder()
        .options(FrameworkOptions {
            commands: vec![ping(), server_info(), reload(), greet(), fruit_command()],
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

    let token = std::env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::DIRECT_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    // This is the fix: `framework` (an `oxcord_framework::Framework`) now
    // satisfies `oxcord::framework::Framework` directly, so it plugs straight
    // into `ClientBuilder::framework()` — no hand-rolled `EventHandler::message`
    // matching needed.
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        eprintln!("Client error: {why:?}");
    }
}
