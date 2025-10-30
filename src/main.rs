use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{RoleId, UserId};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};
use std::num::ParseIntError;

// https://users.rust-lang.org/t/is-there-a-way-to-convert-a-string-to-u64-and-trim-any-floating-points/61574/3
fn integer_part(value: &str) -> Result<u64, ParseIntError> {
    let dot_pos = value.find(".").unwrap_or(value.len());
    value[..dot_pos].parse()
}

struct Handler;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Op a user
#[poise::command(slash_command, prefix_command)]
async fn op(
    ctx: poise::Context<'_, (), Error>,
    #[description = "User"] user: poise::serenity_prelude::Member,
) -> Result<(), Error> {
    let operator_role_id: RoleId = RoleId::new(integer_part(&*std::env::var("OPERATOR_ROLE_ID").expect("missing OPERATOR_ROLE_ID environment variable. use a .env file or set this variable in a script.")).expect("OPERATOR_ROLE_ID is not a valid u64"));

    if !has_permission(&ctx).await? {
        ctx.say("You do not have permission to run this command.").await?;
        return Ok(());
    }

    if user.add_role(ctx.http(), operator_role_id).await.is_ok() {
        ctx.say("Successfully opped ".to_owned() + &user.user.name).await?;
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
    let operator_role_id: RoleId = RoleId::new(integer_part(&*std::env::var("OPERATOR_ROLE_ID").expect("missing OPERATOR_ROLE_ID environment variable. use a .env file or set this variable in a script.")).expect("OPERATOR_ROLE_ID is not a valid u64"));

    if !has_permission(&ctx).await? {
        ctx.say("You do not have permission to run this command.").await?;
        return Ok(());
    }

    if user.remove_role(ctx.http(), operator_role_id).await.is_ok() {
        ctx.say("Successfully de-opped ".to_owned() + &user.user.name).await?;
    } else {
        ctx.say("An error has occurred.").await?;
    }
    Ok(())
}

// Abstracted function to check permissions
async fn has_permission(ctx: &poise::Context<'_, (), Error>) -> Result<bool, Error> {
    if ctx
        .author_member()
        .await
        .ok_or("Could not get member")?
        .roles(ctx.cache())
        .expect("Could not get member roles")
        .contains(
            ctx.guild()
                .unwrap()
                .role_by_name("Operator")
                .expect("Could not get operator role"),
        )
    {
        return Ok(true);
    }

    let allowed_users_env = std::env::var("ALLOWED_USERS").unwrap_or_default();
    let allowed_users: Vec<UserId> = allowed_users_env
        .split(',')
        .filter_map(|id| id.trim().parse::<u64>().ok().map(UserId::new))
        .collect();

    if allowed_users.contains(&ctx.author().id) {
        return Ok(true);
    }

    Ok(false)
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name)
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Copyright (c) 2025 HauC3\n\nLicensed under the Blue Oak Model License 1.0.0 (see https://blueoakcouncil.org/license/1.0.0 for details)\n\n");

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN environment variable. use a .env file or set this variable in a script.");
    let intents = serenity::GatewayIntents::non_privileged();

    // Print allowed users on startup
    let allowed_users_env = std::env::var("ALLOWED_USERS").expect("missing ALLOWED_USERS environment variable. use a .env file or set this variable in a script.");
    let allowed_users: Vec<UserId> = allowed_users_env
        .split(',')
        .filter_map(|id| id.trim().parse::<u64>().ok().map(UserId::new))
        .collect();
    println!("Allowed users: {:?}", allowed_users);

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

    let client = serenity::ClientBuilder::new(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}
