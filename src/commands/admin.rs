use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use crate::ShardManagerContainer;

#[command]
#[help_available(false)]
#[description = "Shutdown the bot"]
#[owners_only]
#[num_args(0)]
pub fn shutdown(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();

    if let Some(manager) = data.get::<ShardManagerContainer>() {
        msg.channel_id.say(&ctx.http, "I'm going down for maintenance!")?;
        manager.lock().shutdown_all();
    } else {
        msg.reply(&ctx.http, "There was a problem getting the shard manager")?;
    };

    Ok(())
}
