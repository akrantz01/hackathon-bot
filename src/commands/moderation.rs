use serenity::framework::standard::{macros::command, ArgError, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::util::{MANAGER_ROLE_ID, MENTOR_ROLE_ID, REPORTS_CHANNEL_ID};

#[command]
#[help_available]
#[description = "Send a report of non-immediate importance"]
#[usage = "<message>"]
#[example = "Help! <username> is being a prick"]
#[num_args(1)]
pub fn report(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get message from args
    let message = match args.single::<String>() {
        Ok(msg) => msg,
        Err(ArgError::Eos) => {
            msg.channel_id
                .say(&ctx.http, "Argument <message> not satisfied")?;
            return Ok(());
        }
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <message>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Retrieve channel
    let channel = ctx
        .http
        .get_channel(REPORTS_CHANNEL_ID.clone())
        .expect("Invalid channel ID");

    // Send message to reports channel
    channel.id().say(
        &ctx.http,
        MessageBuilder::new()
            .mention(&msg.author)
            .push(" reported message '")
            .push(message)
            .push("' from channel #")
            .push(msg.channel_id.name(&ctx.cache).unwrap_or_default()),
    )?;

    // Delete initial message
    msg.delete(&ctx.http)?;

    Ok(())
}

#[command]
#[help_available]
#[description = "Send a report of immediate importance with an optional message. This pings the mods/admins"]
#[usage = "[<message>]"]
#[example = "hey <username> is posting NSFW content"]
#[aliases("em")]
#[min_args(0)]
#[max_args(1)]
pub fn emergency(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get message from args
    let message = match args.single::<String>() {
        Ok(msg) => msg,
        Err(ArgError::Eos) => "".to_string(),
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <message>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Retrieve channel
    let channel = ctx
        .http
        .get_channel(REPORTS_CHANNEL_ID.clone())
        .expect("Invalid channel ID");

    // Send message to reports channel
    channel.id().say(
        &ctx.http,
        MessageBuilder::new()
            .push("(")
            .mention(&RoleId(MANAGER_ROLE_ID.clone()))
            .push(" ")
            .mention(&RoleId(MENTOR_ROLE_ID.clone()))
            .push(") ")
            .push_bold("EMERGENCY!! ")
            .mention(&msg.author)
            .push(" reported an emergency from #")
            .push(msg.channel_id.name(&ctx.cache).unwrap_or_default())
            .push(" with message '")
            .push(message)
            .push("'"),
    )?;

    // Delete initial message
    msg.delete(&ctx.http)?;

    Ok(())
}
