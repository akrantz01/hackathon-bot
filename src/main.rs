#[macro_use]
extern crate lazy_static;

use std::{collections::HashSet, process::exit, sync::Arc};

use dotenv::dotenv;
use log::{error, info};
use serenity::{
    client::bridge::gateway::ShardManager,
    framework::standard::{
        help_commands,
        macros::{group, help},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::{Activity, Ready},
        id::UserId,
    },
    prelude::*,
};

mod commands;
mod data;
mod util;

use commands::{mentors::*, tables::*, admin::*};

// Discord events handler
struct Handler;

impl EventHandler for Handler {
    // Triggers when the client is ready & connected
    fn ready(&self, ctx: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
        ctx.set_activity(Activity::playing("~help"))
    }

    // Triggers when a connection is resumed
    fn resume(&self, ctx: Context, _: ResumedEvent) {
        info!("Successfully reconnected");
        ctx.set_activity(Activity::playing("~help"));
    }
}

// Allow shutting down from command
struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

#[group]
#[commands(join, leave)]
#[description = "Manage your participation in a team"]
struct Tables;

#[group]
#[commands(request, list, complete)]
#[description = "Commands to interact with mentors"]
#[prefixes("m", "mentor")]
struct Mentors;

#[group]
#[commands(shutdown)]
#[description = "Admin only commands"]
#[prefixes("a", "admin")]
struct Admin;

fn main() {
    // Load configuration from a .env file
    // See .env.example for the required fields
    dotenv().ok();

    // Initialize the logger to use environment variables
    // Set RUST_LOG to the minimum level to log at
    env_logger::init();

    // Create client
    let mut client = match Client::new(util::DISCORD_TOKEN.clone(), Handler) {
        Ok(client) => client,
        Err(_) => util::fail("Failed to create the client"),
    };

    // Connect to redis
    data::init(&client);

    // Attach shard manager
    {
        let mut data = client.data.write();
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    // Retrieve the owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        }
        Err(why) => panic!("Couldn't get application info: {:?}", why),
    };

    // Configure the client
    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c.with_whitespace(true)
                    .on_mention(Some(bot_id))
                    .prefix("~")
                    .delimiters(vec![", ", ","])
                    .ignore_bots(true)
                    .owners(owners)
            })
            // Log before execution
            .before(|_, msg, command_name| {
                info!(
                    "Got command '{}' by user '{}'",
                    command_name, msg.author.name
                );
                true
            })
            // Log errors if occurred
            .after(|ctx, msg, command_name, error| {
                // This is currently a work-around to fix some API weirdness
                if command_name == "join" && error.is_err() {
                    match msg.channel_id.say(&ctx.http, "Please run `~join <team_num>` again to confirm creation of team") {
                        Ok(_) => {},
                        Err(e) => error!("Failed to send message: {}", e)
                    };
                    return;
                }

                if let Err(e) = error {
                    error!(
                        "Command '{}' failed for user '{}' with error: {:?}",
                        command_name, msg.author.name, e
                    );
                    match msg.channel_id.say(
                        &ctx.http,
                        &format!("Command '{}' failed: internal server error", command_name),
                    ) {
                        Ok(_) => {}
                        Err(e) => error!("Failed to send message: {}", e),
                    };
                }
            })
            // Log unrecognized commands
            .unrecognised_command(|ctx, msg, unknown_command| {
                info!(
                    "User '{}' attempted to execute an unknown command: {}",
                    msg.author.name, unknown_command
                );

                match msg
                    .channel_id
                    .say(&ctx.http, &format!("Unknown command '{}'", unknown_command))
                {
                    Ok(_) => {}
                    Err(e) => error!("Failed to send message: {}", e),
                };
            })
            // Redirect caller to ~help command
            .prefix_only(|ctx, msg| {
                match msg
                    .channel_id
                    .say(&ctx.http, "Please use `~help` to view the commands")
                {
                    Ok(_) => {}
                    Err(e) => error!("Failed to send message: {}", e),
                };
            })
            // Register command handlers
            .help(&DISPLAY_HELP)
            .group(&TABLES_GROUP)
            .group(&MENTORS_GROUP)
            .group(&ADMIN_GROUP),
    );

    // Attempt to start the client
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
        exit(1);
    }
}

#[help]
#[individual_command_tip = "All command arguments are separated by a `,`.\nTo get help with an individual command, pass its name as an argument to this command."]
#[command_not_found_text = "Command not found: `{}`"]
#[max_levenshtein_distance(3)]
#[indention_prefix = "-"]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
fn display_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
