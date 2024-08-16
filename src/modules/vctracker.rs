use crate::{Context, Error};
use poise::serenity_prelude as serenity;

/// Settings for the VC time tracker
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

/// List the top 10 times in VC.
#[poise::command(
    slash_command,
    guild_only
)]
pub async fn vctop(
    ctx: Context<'_>
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap().get();

    // Grab and sort list
    let times: Vec<(u64, u32)> = sqlx::query!("SELECT user_id, vctrack_total_time FROM users WHERE guild_id = ? ORDER BY vctrack_total_time DESC LIMIT 10", guild_id)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap()
        .iter()
        .map(|r| (r.user_id, r.vctrack_total_time))
        .collect();

    // Build embed
    let mut embed_desc = String::new();

    for (i, user) in times.iter().enumerate() {
        // Get username to display
        let disc_user = serenity::UserId::new(user.0).to_user(ctx.http()).await.unwrap();
        let user_nick = disc_user.nick_in(ctx.http(), guild_id).await;

        let user_nick = if user_nick.is_some() {
            user_nick.unwrap()
        } else if disc_user.global_name.is_some() {
            disc_user.global_name.as_ref().unwrap().to_string()
        } else {
            disc_user.name
        };

        // Format into embed_desc
        embed_desc = format!("{embed_desc}**{}.** {} - {}h {}m {}s\n",
            i + 1,
            user_nick,
            (user.1 / 60) / 60,
            (user.1 / 60) % 60,
            user.1 % 60,
        );
    }

    let mut scoreboard_embed = serenity::CreateEmbed::new()
        .title("VC Time Leaderboard")
        .colour(0xcc3842)
        .description(embed_desc);

    if ctx.guild().unwrap().icon_url().is_some() {
        scoreboard_embed = scoreboard_embed.thumbnail(ctx.guild().unwrap().icon_url().unwrap());
    }

    ctx.send(poise::CreateReply::default().embed(scoreboard_embed)).await?;

    Ok(())
}