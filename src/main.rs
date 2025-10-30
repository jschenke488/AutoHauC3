// Import necessary crates and modules
use dotenv::dotenv;
use log::{error, info};
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{RoleId, UserId};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};
use once_cell::sync::Lazy;
use tokio::task;

// https://users.rust-lang.org/t/is-there-a-way-to-convert-a-string-to-u64-and-trim-any-floating-points/61574/3
// Function to extract the integer part of a string, with error handling
// Parses environment variables that may contain floating-point values
fn integer_part(value: &str) -> Result<u64, String> {
    let dot_pos = value.find('.').unwrap_or(value.len());
    value[..dot_pos]
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse integer part from '{}': {}", value, e))
}

// Define the bot's event handler
struct Handler;

// Define a custom error type for the bot
// This allows for more flexible error handling
type Error = Box<dyn std::error::Error + Send + Sync>;

/// Op a user
#[poise::command(slash_command, prefix_command)]
async fn op(
    ctx: poise::Context<'_, (), Error>,
    #[description = "User"] user: poise::serenity_prelude::Member,
) -> Result<(), Error> {
    // Check if the user has permission to run this command
    if !has_permission(&ctx).await? {
        ctx.say("You do not have permission to run this command.").await?;
        return Ok(());
    }

    // Attempt to add the operator role to the specified user
    if user.add_role(ctx.http(), *OPERATOR_ROLE_ID).await.is_ok() {
        ctx.say(format!("Successfully opped {}", user.user.name)).await?;
    } else {
        ctx.say("An error has occurred.").await?;
    }
    Ok(())
}

/// De-op a user
#[poise::command(slash_command, prefix_command)]
async fn deop(
    ctx: poise::Context<'_, (), Error>,
    #[description = "User"] user: poise::serenity_prelude::Member,
) -> Result<(), Error> {
    // Check if the user has permission to run this command
    if !has_permission(&ctx).await? {
        ctx.say("You do not have permission to run this command.").await?;
        return Ok(());
    }

    // Attempt to remove the operator role from the specified user
    if user.remove_role(ctx.http(), *OPERATOR_ROLE_ID).await.is_ok() {
        ctx.say(format!("Successfully de-opped {}", user.user.name)).await?;
    } else {
        ctx.say("An error has occurred.").await?;
    }
    Ok(())
}

// Helper function to fetch environment variables with error handling
fn get_env_var(key: &str) -> Result<String, String> {
    std::env::var(key).map_err(|_| format!("Missing {} environment variable. Use a .env file or set this variable in a script.", key))
}

// Function to check if a user has the necessary permissions
// This checks both the user's roles and their presence in the allowed users list
async fn has_permission(ctx: &poise::Context<'_, (), Error>) -> Result<bool, Error> {
    let member = ctx.author_member().await.ok_or_else(|| {
        let err = "Could not get member";
        error!("{}", err);
        err
    })?;

    let roles = member.roles(ctx.cache()).ok_or_else(|| {
        let err = "Could not get member roles";
        error!("{}", err);
        err
    })?;

    if let Some(guild) = ctx.guild() {
        if let Some(operator_role) = guild.role_by_name(&OPERATOR_ROLE_NAME) {
            if roles.contains(operator_role) {
                return Ok(true);
            }
        }
    }

    if ALLOWED_USERS.contains(&ctx.author().id) {
        return Ok(true);
    }

    Ok(false)
}

// Define global constants using once_cell::sync::Lazy
// These are initialized once and used throughout the program
static ALLOWED_USERS: Lazy<Vec<UserId>> = Lazy::new(|| {
    task::block_in_place(|| {
        let allowed_users_env = std::env::var("ALLOWED_USERS")
            .expect("missing ALLOWED_USERS environment variable. use a .env file or set this variable in a script.");
        allowed_users_env
            .split(',')
            .filter_map(|id| id.trim().parse::<u64>().ok().map(UserId::new))
            .collect()
    })
});

static OPERATOR_ROLE_NAME: Lazy<String> = Lazy::new(|| {
    task::block_in_place(|| {
        std::env::var("OPERATOR_ROLE_NAME").unwrap_or_else(|_| "Operator".to_string())
    })
});

static OPERATOR_ROLE_ID: Lazy<RoleId> = Lazy::new(|| {
    task::block_in_place(|| {
        let operator_role_id_env = std::env::var("OPERATOR_ROLE_ID")
            .expect("missing OPERATOR_ROLE_ID environment variable. use a .env file or set this variable in a script.");
        RoleId::new(
            integer_part(&operator_role_id_env)
                .expect("Invalid OPERATOR_ROLE_ID: must be a valid u64"),
        )
    })
});

// Event handler implementation for the bot
#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name)
    }
}

// Main function to start the bot
// This initializes the environment, sets up the framework, and starts the client
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Set a default logging level if RUST_LOG is not defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info,my_crate=info,serenity=warn,poise=warn,tracing::span=off");
    }

    env_logger::init(); // Initialize the logger

    info!("Copyright (c) 2025 HauC3\n\nLicensed under the Blue Oak Model License 1.0.0 (see https://blueoakcouncil.org/license/1.0.0 for details)\n\n");

    let token = match get_env_var("DISCORD_TOKEN") {
        Ok(token) => token,
        Err(err) => {
            error!("{}", err);
            return;
        }
    };

    let intents = serenity::GatewayIntents::non_privileged();

    // Log allowed users on startup
    info!("Allowed users: {:?}", *ALLOWED_USERS);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![op(), deop()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(())
            })
        })
        .build();

    let mut client = match serenity::ClientBuilder::new(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
    {
        Ok(client) => client,
        Err(e) => {
            error!("Failed to create client: {}", e);
            return;
        }
    };

    if let Err(e) = client.start().await {
        error!("Client encountered an error: {}", e);
    }
}
