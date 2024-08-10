use crate::{log, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    subcommands("ignorechannel")
)]
pub async fn vctracker(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn ignorechannel(
    ctx: Context<'_>,
    channel: Option<serenity::Channel>,
) -> Result<(), Error> {
    let query_channel = match &channel {
        Some(c) => c.id().get().to_string(),
        None => String::from("NULL"),
    };

    sqlx::query!("UPDATE guild_settings SET vctrack_ignored_channel = ? WHERE guild_id = ?", query_channel, ctx.guild_id().unwrap().get())
        .execute(&ctx.data().database)
        .await
        .unwrap();

    match &channel {
        Some(c) => {
            ctx.say(format!("Channel {} will be ignored for tracking time spent in VC.", c)).await?;
        },
        None => {
            ctx.say("No longer ignoring any channels for tracking time spent in VC.").await?;
        }
    }

    Ok(())
}