use oxcord::framework::standard::macros::command;
use oxcord::framework::standard::{Args, CommandResult};
use oxcord::model::prelude::*;
use oxcord::prelude::*;

#[command]
pub async fn multiply(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let one = args.single::<f64>()?;
    let two = args.single::<f64>()?;

    let product = one * two;

    msg.channel_id.say(&ctx.http, product.to_string()).await?;

    Ok(())
}
