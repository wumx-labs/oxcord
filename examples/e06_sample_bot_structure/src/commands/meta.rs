use oxcord::framework::standard::macros::command;
use oxcord::framework::standard::CommandResult;
use oxcord::model::prelude::*;
use oxcord::prelude::*;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}
