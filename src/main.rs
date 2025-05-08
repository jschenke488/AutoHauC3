use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{RoleId, UserId};
use serenity::async_trait;
use serenity::model::gateway::Ready;
use serenity::prelude::{Context, EventHandler};

struct Handler;

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Op a user
#[poise::command(slash_command, prefix_command)]
async fn op(
    ctx: poise::Context<'_, (), Error>,
    #[description = "User"] user: poise::serenity_prelude::Member,
) -> Result<(), Error> {
    let mut has_op = false;
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
        has_op = true;
    }
    /* 209419219899514880 Jay
    326839304829665282 Camden
    675130001389125662 Trin */
    if ctx.author().id == UserId::new(209419219899514880)
        || ctx.author().id == UserId::new(326839304829665282)
        || ctx.author().id == UserId::new(675130001389125662)
    {
        has_op = true;
    }
    if !has_op {
        ctx.say("You do not have permission to run this command.")
            .await?;
    } else if user
        .add_role(ctx.http(), RoleId::new(718954921353019454))
        .await
        .is_ok()
    {
        ctx.say("Successfully opped ".to_owned() + &user.user.name)
            .await?;
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
    let mut has_op = false;
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
        has_op = true;
    }
    /* 209419219899514880 Jay
    326839304829665282 Camden
    675130001389125662 Trin */
    if ctx.author().id == UserId::new(209419219899514880)
        || ctx.author().id == UserId::new(326839304829665282)
        || ctx.author().id == UserId::new(675130001389125662)
    {
        has_op = true;
    }
    if !has_op {
        ctx.say("You do not have permission to run this command.")
            .await?;
    } else if user
        .remove_role(ctx.http(), RoleId::new(718954921353019454))
        .await
        .is_ok()
    {
        ctx.say("Successfully de-opped ".to_owned() + &user.user.name)
            .await?;
    } else {
        ctx.say("An error has occurred.").await?;
    }
    Ok(())
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
