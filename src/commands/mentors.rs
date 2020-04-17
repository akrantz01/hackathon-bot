use chrono::prelude::*;
use log::error;
use redis::Commands;
use serenity::framework::standard::{macros::command, ArgError, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::data::{add_help_request, get_connection, get_help_request};
use crate::util::{MENTORS_CHANNEL_ID, MENTOR_ROLE_ID};

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
    let mut client = get_connection(&ctx.data)?;

    // Retrieve team from database
    let team = match client.hget::<&str, u64, Option<String>>("tables", msg.author.id.0)? {
        Some(team) => team,
        None => format!("{}#{}", &msg.author.name, &msg.author.discriminator),
    };

    // Get the current time
    let current_time: DateTime<Local> = Local::now();

    // Set values
    add_help_request(
        &mut client,
        description,
        link,
        team.clone(),
        current_time.timestamp_millis(),
    )?;

    // Send confirmation
    msg.channel_id.say(
        &ctx.http,
        MessageBuilder::new()
            .push("Successfully requested help for ")
            .mention(&msg.author)
            .push("."),
    )?;

    // Retrieve channel
    let channel = ctx
        .http
        .get_channel(MENTORS_CHANNEL_ID.clone())
        .expect("Invalid channel ID");

    // Send notification to mentors
    channel.id().say(
        &ctx.http,
        MessageBuilder::new()
            .push("New help request from ")
            .mention(&msg.author)
            .push(if team.contains("Table ") {
                format!(" in {}", team)
            } else {
                String::new()
            })
            .build(),
    )?;

    Ok(())
}

#[command]
#[help_available]
#[description = "List all help requests"]
#[num_args(0)]
pub fn list(ctx: &mut Context, msg: &Message, _: Args) -> CommandResult {
    // Check if current user is a mentor
    if !msg
        .author
        .has_role(&ctx.http, msg.guild_id.unwrap(), *MENTOR_ROLE_ID)?
    {
        msg.channel_id.say(
            &ctx.http,
            MessageBuilder::new()
                .mention(&msg.author)
                .push(" You must be a mentor to run this command!")
                .build(),
        )?;
        return Ok(());
    }

    // Retrieve redis connection
    let mut client = get_connection(&ctx.data)?;

    // Get all requests
    let requests: redis::Iter<String> = client.scan_match("help_request:*")?;

    // Get a new client
    let mut client = get_connection(&ctx.data)?;

    // Send table of help requests
    msg.channel_id.send_message(&ctx.http, |m| {
        m.embed(|e| {
            e.description("Here is a list of all the uncompleted help requests:");

            // Format list
            for key in requests {
                // Retrieve data
                let (desc, link, table, ts) = match get_help_request(&mut client, &key) {
                    Ok(data) => data,
                    Err(e) => {
                        error!("Failed to query Redis: {}", e);
                        (String::new(), String::new(), String::new(), 0)
                    }
                };

                // Ignore invalid records
                if ts == 0 {
                    continue;
                }

                e.field(
                    key.get(13..21).unwrap(),
                    format!(
                        "**Timestamp**: {}\n**Description**: {}\n**Link**: {}\n**For**: {}",
                        Local.timestamp(ts / 1000, 0).to_string(),
                        desc,
                        link,
                        table
                    ),
                    true,
                );
            }

            e
        })
    })?;

    Ok(())
}

#[command]
#[help_available]
#[description = "Mark a help request as completed"]
#[usage = "<id>"]
#[example = "abcd1234"]
#[num_args(1)]
pub fn complete(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Check if current user is a mentor
    if !msg
        .author
        .has_role(&ctx.http, msg.guild_id.unwrap(), *MENTOR_ROLE_ID)?
    {
        msg.channel_id.say(
            &ctx.http,
            MessageBuilder::new()
                .mention(&msg.author)
                .push(" You must be a mentor to run this command!")
                .build(),
        )?;
        return Ok(());
    }

    // Get request id
    let id = match args.single::<String>() {
        Ok(id) => id,
        Err(ArgError::Eos) => {
            msg.channel_id
                .say(&ctx.http, "Argument <id> not satisfied")?;
            return Ok(());
        }
        Err(ArgError::Parse(why)) => {
            msg.channel_id
                .say(&ctx.http, format!("Failed parsing argument <id>: {}", why))?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Retrieve connection and delete
    let mut client = get_connection(&ctx.data)?;
    client.del(format!("help_request:{}", &id))?;

    // Send confirmation
    msg.channel_id.say(
        &ctx.http,
        MessageBuilder::new()
            .push("Deleted help request ")
            .push(&id)
            .push(" for ")
            .mention(&msg.author)
            .push("."),
    )?;

    Ok(())
}
