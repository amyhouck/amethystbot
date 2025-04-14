use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    guild_only,
    subcommands("setmessage", "setimage", "setchannel"),
    member_cooldown = 5,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn boost(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the boost message in the embed.
#[poise::command(
    slash_command,
    check = "boost_channel_check"
)]
pub async fn setmessage(
    ctx: Context<'_>,
    #[description = "The message to send when a user boosts a server."] message: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if message.is_none() {
        sqlx::query!("UPDATE boost SET message = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer sending a custom boost message!", ctx.author())).await?;
        return Ok(());
    }

    // Grab channel ID and query DB
    sqlx::query!("UPDATE boost SET message = ? WHERE guild_id = ?", message.as_ref().unwrap(), guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, set the custom boost message to: ```{}```", ctx.author(), message.unwrap())).await?;
    Ok(())
}

/// Set the image to appear in the boost embed.
#[poise::command(
    slash_command,
    check = "boost_channel_check"
)]
pub async fn setimage(
    ctx: Context<'_>,
    #[description = "The URL of an image to include with the custom boost message."] image_url: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if image_url.is_none() {
        sqlx::query!("UPDATE boost SET image_url = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer including an image in the boost messages!", ctx.author())).await?;
        return Ok(());
    }

    // Query DB
    sqlx::query!("UPDATE boost SET image_url = ? WHERE guild_id = ?", image_url.as_ref().unwrap(), guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, image for the boost messages set to {}", ctx.author(), image_url.unwrap())).await?;
    Ok(())
}

/// Set the channel to send boost messages to.
#[poise::command(slash_command)]
pub async fn setchannel(
    ctx: Context<'_>,
    #[description = "The channel to send the boost messages to."] channel: Option<serenity::Channel>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if channel.is_none() {
        sqlx::query!("UPDATE boost SET channel_id = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer sending boost messages!", ctx.author())).await?;
        return Ok(());
    }

    // Grab channel ID and query DB
    let channel_id = channel.as_ref().unwrap().id().get();

    sqlx::query!("UPDATE boost SET channel_id = ? WHERE guild_id = ?", channel_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, now sending boost messages to {}!", ctx.author(), channel.unwrap())).await?;
    Ok(())
}

//--------------------
// Function library
//--------------------
// Check to make sure channel is set
async fn boost_channel_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    let channel = sqlx::query!("SELECT channel_id FROM boost WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    match channel.channel_id {
        Some(_) => Ok(true),
        None => {
            ctx.send(poise::CreateReply::default()
                .content("You must select a channel to post boost messages in before using this command!")
                .ephemeral(true)
            ).await?;
            Ok(false)
        }
    }
}