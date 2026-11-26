use std::env;

use dotenv::dotenv;
use oxcord::async_trait;
use oxcord::builder::{
    CreateButton,
    CreateContainer,
    CreateInteractionResponse,
    CreateInteractionResponseMessage,
    CreateMessage,
    CreateSection,
    CreateSeparator,
    CreateTextDisplay,
    CreateThumbnail,
};
use oxcord::model::application::{Interaction, SeparatorSpacing};
use oxcord::model::channel::Message;
use oxcord::model::gateway::Ready;
use oxcord::model::id::ChannelId;
use oxcord::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == "!v2" {
            let channel_id: ChannelId = msg.channel_id;

            // Build a modern Components V2 message:
            // - A colored Container card
            // - Header markdown TextDisplay
            // - Separator divider
            // - Section with text and a Button accessory
            // - Thumbnail image
            let header = CreateTextDisplay::new(
                "## 🚀 Welcome to OXCORD Components V2!\nThis card is built using modern Discord Components V2.",
            );

            let separator = CreateSeparator::new().divider(true).spacing(SeparatorSpacing::Small);

            let click_btn = CreateButton::new("v2_action_button").label("Try V2 Button");

            let section = CreateSection::new()
                .add_text("Click the button on the right to interact with Components V2:")
                .accessory(click_btn);

            let thumbnail = CreateThumbnail::new("https://raw.githubusercontent.com/rust-lang/rust-artwork/master/logo/rust-logo-512x512.png")
                .description("Rust Logo");

            let container = CreateContainer::new()
                .accent_color(0x5865F2) // Discord Blurple
                .add_component(header)
                .add_component(separator)
                .add_component(section)
                .add_component(thumbnail);

            // components_v2() automatically sets MessageFlags::IS_COMPONENTS_V2!
            let builder = CreateMessage::new()
                .content("Here is your Components V2 card:")
                .components_v2(vec![container.into()]);

            if let Err(why) = channel_id.send_message(&ctx.http, builder).await {
                println!("Error sending message: {why:?}");
            }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Component(component) = interaction
            && component.data.custom_id == "v2_action_button"
        {
            let response = CreateInteractionResponseMessage::new()
                .content("🎉 You clicked the Components V2 button!")
                .ephemeral(true);

            if let Err(why) = component
                .create_response(&ctx.http, CreateInteractionResponse::Message(response))
                .await
            {
                println!("Error responding to component interaction: {why:?}");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected and ready for Components V2 testing!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
