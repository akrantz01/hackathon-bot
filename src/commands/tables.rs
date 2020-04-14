use redis::Commands;
use serenity::framework::standard::{macros::command, ArgError, Args, CommandError, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

use crate::data::get_connection;
use crate::util::{
    random_color, BOT_ROLE_ID, EVERYONE_ROLE_ID, MANAGER_ROLE_ID, MENTOR_ROLE_ID,
    TABLES_CATEGORY_ID, TEAMLESS_ROLE_ID,
};

#[command]
#[help_available]
#[description = "Add yourself to a table"]
#[usage = "<table_number>"]
#[example = "1"]
#[num_args(1)]
pub fn join(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Parse team number from args
    let team_num = match args.single::<i64>() {
        Ok(num) => num,
        Err(ArgError::Eos) => {
            msg.channel_id
                .say(&ctx.http, "Argument <team_number> not satisifed")?;
            return Ok(());
        }
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <team_number>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Check if current user part of team
    if !msg
        .author
        .has_role(&ctx.http, msg.guild_id.unwrap(), *TEAMLESS_ROLE_ID)?
    {
        msg.channel_id.say(
            &ctx.http,
            MessageBuilder::new()
                .mention(&msg.author)
                .push(" You're already part of a team!")
                .build(),
        )?;
        return Ok(());
    }

    // Retrieve guild
    let guild = msg.guild(&ctx.cache).unwrap();

    // Create role if not exists
    let role = match guild.read().role_by_name(&format!("Table {}", team_num)) {
        Some(role) => role.clone(),
        None => guild.read().create_role(&ctx.http, |r| {
            r.name(format!("Table {}", team_num))
                .colour(random_color().0.into())
                .mentionable(false)
                .hoist(true)
                .position(3)
                .permissions(
                    Permissions::CHANGE_NICKNAME
                        | Permissions::READ_MESSAGES
                        | Permissions::SEND_MESSAGES
                        | Permissions::EMBED_LINKS
                        | Permissions::ATTACH_FILES
                        | Permissions::READ_MESSAGE_HISTORY
                        | Permissions::USE_EXTERNAL_EMOJIS
                        | Permissions::ADD_REACTIONS
                        | Permissions::CONNECT
                        | Permissions::SPEAK
                        | Permissions::USE_VAD,
                )
        })?,
    };

    // Add current user to role and remove from teamless role
    let mut member = guild.read().member(&ctx.http, msg.author.id)?;
    member.add_role(&ctx.http, role.id)?;
    member.remove_role(&ctx.http, *TEAMLESS_ROLE_ID)?;

    // Create table if exists
    if guild
        .read()
        .channel_id_from_name(&ctx.cache, format!("table-{}", team_num))
        .is_none()
    {
        guild.read().create_channel(&ctx.http, |c| {
            c.name(format!("table-{}", team_num))
                .kind(ChannelType::Text)
                .topic(format!("Private discussion space for Table {}", team_num))
                .category(*TABLES_CATEGORY_ID)
                .nsfw(false)
                .permissions(vec![
                    PermissionOverwrite {
                        kind: PermissionOverwriteType::Role(role.id),
                        allow: Permissions::READ_MESSAGES
                            | Permissions::READ_MESSAGE_HISTORY
                            | Permissions::SEND_MESSAGES,
                        deny: Permissions::empty(),
                    },
                    PermissionOverwrite {
                        kind: PermissionOverwriteType::Role(RoleId(*EVERYONE_ROLE_ID)),
                        allow: Permissions::empty(),
                        deny: Permissions::READ_MESSAGES | Permissions::READ_MESSAGE_HISTORY,
                    },
                    PermissionOverwrite {
                        kind: PermissionOverwriteType::Role(RoleId(*MENTOR_ROLE_ID)),
                        allow: Permissions::READ_MESSAGES
                            | Permissions::READ_MESSAGE_HISTORY
                            | Permissions::SEND_MESSAGES,
                        deny: Permissions::empty(),
                    },
                    PermissionOverwrite {
                        kind: PermissionOverwriteType::Role(RoleId(*MANAGER_ROLE_ID)),
                        allow: Permissions::READ_MESSAGES
                            | Permissions::READ_MESSAGE_HISTORY
                            | Permissions::SEND_MESSAGES
                            | Permissions::MANAGE_MESSAGES,
                        deny: Permissions::empty(),
                    },
                    PermissionOverwrite {
                        kind: PermissionOverwriteType::Role(RoleId(*BOT_ROLE_ID)),
                        allow: Permissions::READ_MESSAGES | Permissions::READ_MESSAGE_HISTORY,
                        deny: Permissions::empty(),
                    },
                ])
        })?;
    }

    // Retrieve redis connection to persistently cache user's team
    let mut client = get_connection(&ctx.data)?;
    client.hset("tables", msg.author.id.0, format!("Table {}", team_num))?;

    // Send confirmation message
    msg.channel_id.say(
        &ctx.http,
        MessageBuilder::new()
            .push("Successfully added ")
            .mention(&msg.author)
            .push(" to ")
            .push_mono(format!("Table {}", team_num))
            .push(".")
            .build(),
    )?;

    Ok(())
}

#[command]
#[help_available]
#[description = "Remove yourself from the table you're in"]
#[usage = "<table_number>"]
#[example = "1"]
#[num_args(1)]
pub fn leave(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // Parse team number from args
    let team_num = match args.single::<i64>() {
        Ok(num) => num,
        Err(ArgError::Eos) => {
            msg.channel_id
                .say(&ctx.http, "Argument <team_number> not satisfied")?;
            return Ok(());
        }
        Err(ArgError::Parse(why)) => {
            msg.channel_id.say(
                &ctx.http,
                format!("Failed parsing argument <team_number>: {}", why),
            )?;
            return Ok(());
        }
        Err(e) => return Err(CommandError(e.to_string())),
    };

    // Check user has a team in general
    if msg
        .author
        .has_role(&ctx.http, msg.guild_id.unwrap(), *TEAMLESS_ROLE_ID)?
    {
        msg.channel_id.say(
            &ctx.http,
            MessageBuilder::new()
                .mention(&msg.author)
                .push(" You're not part of a team!")
                .build(),
        )?;
        return Ok(());
    }

    // Retrieve guild
    let guild = msg.guild(&ctx.cache).unwrap();

    // Retrieve role by name
    let role = match guild.read().role_by_name(&format!("Table {}", team_num)) {
        Some(role) => role.clone(),
        None => {
            msg.channel_id.say(
                &ctx.http,
                MessageBuilder::new()
                    .mention(&msg.author)
                    .push(" You're not part of 'Table ")
                    .push(team_num)
                    .push("'!")
                    .build(),
            )?;
            return Ok(());
        }
    };

    // Remove user from role and add teamless role
    let mut member = guild.read().member(&ctx.http, msg.author.id)?;
    member.remove_role(&ctx.http, role.id)?;
    member.add_role(&ctx.http, *TEAMLESS_ROLE_ID)?;

    // Remove user's team from redis cache
    let mut client = get_connection(&ctx.data)?;
    client.hdel("tables", msg.author.id.0)?;

    // Send confirmation message
    msg.channel_id.say(
        &ctx.http,
        MessageBuilder::new()
            .push("Successfully removed ")
            .mention(&msg.author)
            .push(" from ")
            .push_mono(format!("Table {}", team_num))
            .push(".")
            .build(),
    )?;

    Ok(())
}
