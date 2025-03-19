use crate::{Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    guild_only,
    subcommands("setmessage", "setimage", "setchannel"),
    member_cooldown = 5,
)]
pub async fn welcome(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the welcome message in the embed.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "welcome_channel_check"
)]
pub async fn setmessage(
    ctx: Context<'_>,
    #[description = "The message to send when a user joins."] message: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if message.is_none() {
        sqlx::query!("UPDATE welcome SET message = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer including a custom message in the welcome messages!", ctx.author())).await?;
        return Ok(());
    }

    // Grab channel ID and query DB
    sqlx::query!("UPDATE welcome SET message = ? WHERE guild_id = ?", message.as_ref().unwrap(), guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, set the custom welcome message to: ```{}```", ctx.author(), message.unwrap())).await?;
    Ok(())
}

/// Set the image to appear in the welcome embed.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "welcome_channel_check"
)]
pub async fn setimage(
    ctx: Context<'_>,
    #[description = "The URL of an image to include with the welcome message."] image_url: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if image_url.is_none() {
        sqlx::query!("UPDATE welcome SET image_url = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer including an image in the welcome messages!", ctx.author())).await?;
        return Ok(());
    }

    // Query DB
    sqlx::query!("UPDATE welcome SET image_url = ? WHERE guild_id = ?", image_url.as_ref().unwrap(), guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, image for the welcome messages set to {}", ctx.author(), image_url.unwrap())).await?;
    Ok(())
}

/// Set the channel to send welcome messages to.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn setchannel(
    ctx: Context<'_>,
    #[description = "The channel to send the welcome messages to."] channel: Option<serenity::Channel>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Handle empty channel (disables messaging)
    if channel.is_none() {
        sqlx::query!("UPDATE welcome SET channel_id = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("{}, no longer sending welcome messages!", ctx.author())).await?;
        return Ok(());
    }

    // Grab channel ID and query DB
    let channel_id = channel.as_ref().unwrap().id().get();

    sqlx::query!("UPDATE welcome SET channel_id = ? WHERE guild_id = ?", channel_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, now sending welcome messages to {}!", ctx.author(), channel.unwrap())).await?;
    Ok(())
}

/// Set channel to send member leave notifications.
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    guild_only,
)]
pub async fn setleavechannel(
    ctx: Context<'_>,
    #[description = "The channel to send a message when a user leaves the server."] channel: Option<serenity::Channel>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Set the channel to send leave announcements to
    if channel.is_some() {
        let channel_id = channel.as_ref().unwrap().id().get();

        sqlx::query!("UPDATE guild_settings SET member_leave_channel_id = ? WHERE guild_id = ?", channel_id, guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("Now sending a message when a server member leaves to {}!", channel.unwrap())).await?;
    } else {
        sqlx::query!("UPDATE guild_settings SET member_leave_channel_id = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say("No longer announcing when a server member leaves!").await?;
    }

    Ok(())
}

//--------------------
// Function library
//--------------------
// Check to make sure channel is set
async fn welcome_channel_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    let channel = sqlx::query!("SELECT channel_id FROM welcome WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    match channel.channel_id {
        Some(_) => Ok(true),
        None => {
            ctx.send(poise::CreateReply::default()
                .content("You must select a channel to post welcome messages in before using this command!")
                .ephemeral(true)
            ).await?;
            Ok(false)
        }
    }
}