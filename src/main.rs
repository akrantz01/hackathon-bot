use std::{
    collections::HashSet,
    env,
    process::exit,
};

use dotenv::dotenv;
use log::{error, info};
use serenity::{
    framework::standard::{
        Args, CheckResult, CommandOptions, CommandResult, CommandGroup,
        DispatchError, HelpOptions, help_commands, StandardFramework,
        macros::{command, group, help, check},
    },
    model::{channel::{Channel, Message}, gateway::Ready, id::{ChannelId, UserId}},
    utils::{content_safe, ContentSafeOptions},
    prelude::*,
};

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);
    }
}

fn main() {
    // Load configuration from a .env file
    // See .env.example for the required fields
    dotenv().ok();

    // Initialize the logger to use environment variables
    // Set RUST_LOG to the minimum level to log at
    env_logger::init();

    // Retrieve token
    let token = match env::var("DISCORD_TOKEN") {
         Ok(token) => token,
         Err(_) => fail("No bot token present! Exiting...")
    };

    // Create client
    let mut client = match Client::new(&token, Handler) {
        Ok(client) => client,
        Err(_) => fail("Failed to create the client")
    };

    // Retrieve the owners and id
    let (owners, bot_id) = match client.cache_and_http.http.get_current_application_info() {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);
            (owners, info.id)
        },
        Err(why) => panic!("Couldn't get application info: {:?}", why)
    };

    // Configure the client
    client.with_framework(StandardFramework::new()
        .configure(|c| c
            .with_whitespace(true)
            .on_mention(Some(bot_id))
            .prefix("!")
            .delimiters(vec![", ", ","])
            .ignore_bots(true)
            .owners(owners))
            // Log before execution
            .before(|_, msg, command_name| {
                info!("Got command '{}' by user '{}'", command_name, msg.author.name);
                true
            })
            // Log errors if occurred
            .after(|_, msg, command_name, error| {
                if let Err(e) = error {
                    error!("Command '{}' failed for user '{}' with error: {:?}", command_name, msg.author.name, e);
                }
            })
            // Log unrecognized commands
            .unrecognised_command(|ctx, msg, unknown_command| {
                info!("User '{}' attempted to execute an unknown command: {}", msg.author.name, unknown_command);

                match msg.channel_id.say(&ctx.http, &format!("Unknown command '{}'", unknown_command)) {
                    Ok(_) => {},
                    Err(e) => error!("Failed to send message: {}", e)
                };
            })
            // Register command handlers
            .help(&DISPLAY_HELP)
        );

    // Attempt to start the client
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
        exit(1);
    }
}


/// Print an error and exit with error code
fn fail(prompt: &'_ str) -> ! {
    error!("{}", prompt);
    exit(1);
}

#[help]
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
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}
