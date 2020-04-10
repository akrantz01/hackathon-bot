use std::time::{SystemTime, UNIX_EPOCH};

use redis::Commands;
use serenity::framework::standard::{macros::command, ArgError, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::util::MENTOR_ROLE_ID;

#[command]
#[help_available]
#[description = "Request help from a mentor"]
#[usage = "<description> [<link to code>]"]
#[example = "some description, https://github.com/test/test"]
#[min_args(1)]
#[max_args(2)]
pub fn request(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Get description from args
    let description = match args.single::<String>() {
        Ok(desc) => desc,
        Err(ArgError::Eos) => {
            msg.channel_id
                .say(&ctx.http, "Argument <description> not satisfied")?;
            return Ok(());
        }
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <description>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Get link to code from args
    let link = match args.single::<String>() {
        Ok(l) => l,
        Err(ArgError::Eos) => String::new(),
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <link to code>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Retrieve redis connection
    let data = ctx.data.read();
    let client = data.get::<crate::RedisConnection>().expect("Expected RedisConnection in ShareMap.");
    let mut connection = client.get_connection()?;

    // Get the current time
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");

    // Set values
    connection.hset("descriptions", since_epoch.as_secs(), description)?;
    connection.hset("links", since_epoch.as_secs(), link)?;

    // Send confirmation
    msg.channel_id.say(&ctx.http, MessageBuilder::new().push("Help requested for ").mention(&msg.author).push("."))?;

    Ok(())
}
