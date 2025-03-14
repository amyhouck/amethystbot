use crate::{log, Context, Error};
use poise::serenity_prelude as serenity;

// VCTop Leaderboard Types
#[derive(poise::ChoiceParameter)]
enum VCTopType {
    All,
    Monthly
}

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
    #[description = "The VC channel to ignore for tracking time."] channel: Option<serenity::Channel>,
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
    ctx: Context<'_>,
    #[rename = "type"]
    #[description = "The timeframe of the leaderboard you want to see."] leaderboard_type: Option<VCTopType>,
) -> Result<(), Error> {
    let leaderboard_type = leaderboard_type.unwrap_or(VCTopType::All);

    // Grab guild info and recheck voice times
    let guild_id = ctx.guild_id().unwrap().get();

    let ctx_voice_states = ctx.guild().unwrap().voice_states.clone();
    let mut voice_states: Vec<serenity::VoiceState> = Vec::new();

    for (_, v) in ctx_voice_states {
        voice_states.push(v);
    }
    
    let futures = voice_states.iter().map(|vs| recheck_time(vs, &ctx.data().database));
    futures::future::join_all(futures).await;

    // Grab and sort list
    let times: Vec<(String, u32)> = match leaderboard_type {
        VCTopType::All => {
            sqlx::query!("SELECT display_name, vctrack_total_time FROM users WHERE guild_id = ? ORDER BY vctrack_total_time DESC LIMIT 10", guild_id)
                .fetch_all(&ctx.data().database)
                .await
                .unwrap()
                .iter()
                .map(|r| (r.display_name.to_string(), r.vctrack_total_time))
                .collect()
        },
        VCTopType::Monthly => {
            sqlx::query!("SELECT display_name, vctrack_monthly_time FROM users WHERE guild_id = ? ORDER BY vctrack_monthly_time DESC LIMIT 10", guild_id)
                .fetch_all(&ctx.data().database)
                .await
                .unwrap()
                .iter()
                .map(|r| (r.display_name.to_string(), r.vctrack_monthly_time))
                .collect()
        }
    };

    // Build embed
    let mut embed_desc = String::new();

    for (i, user) in times.iter().enumerate() {
        // Format into embed_desc
        embed_desc = format!("{embed_desc}**{}.** {} - {}h {}m {}s\n",
            i + 1,
            user.0,
            (user.1 / 60) / 60,
            (user.1 / 60) % 60,
            user.1 % 60,
        );
    }

    let title = match leaderboard_type {
        VCTopType::All => "VC All-time Leaderboard",
        VCTopType::Monthly => "VC Monthly Leaderboard",
    };

    let mut scoreboard_embed = serenity::CreateEmbed::new()
        .title(title)
        .colour(0xcc3842)
        .description(embed_desc);

    if ctx.guild().unwrap().icon_url().is_some() {
        scoreboard_embed = scoreboard_embed.thumbnail(ctx.guild().unwrap().icon_url().unwrap());
    }

    ctx.send(poise::CreateReply::default().embed(scoreboard_embed)).await?;

    Ok(())
}

// Recheck VC participant time
pub async fn recheck_time(
    voice_state: &serenity::VoiceState,
    database: &sqlx::MySqlPool
) -> Result<(), Error> {
    // Grab necessary info
    let user_id = voice_state.user_id.get();

    let guild_id = match voice_state.guild_id {
        Some(id) => id.get(),
        None => return Err("Unable to retrieve voice state information from Discord.".into())
    };

    let channel_id = voice_state.channel_id.unwrap().get();
    let ignored_channel = sqlx::query!("SELECT vctrack_ignored_channel FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(database)
        .await
        .unwrap()
        .vctrack_ignored_channel
        .unwrap_or(0);

    // If channel_id isn't ignored, update times
    if channel_id != ignored_channel {
        // Skip if join time is 0
        let join_time = sqlx::query!("SELECT vctrack_join_time FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .fetch_one(database)
            .await
            .unwrap()
            .vctrack_join_time;

        if join_time == 0 {
            log::write_log(log::LogType::VCTrackerSafeguardSkip { guild_id, user_id });
        } else {
            // Update users time
            let query = format!("
                UPDATE users SET vctrack_total_time = (UNIX_TIMESTAMP() - vctrack_join_time) + vctrack_total_time WHERE guild_id = {guild_id} AND user_id = {user_id};
                UPDATE users SET vctrack_monthly_time = vctrack_monthly_time + (UNIX_TIMESTAMP() - vctrack_join_time) WHERE guild_id = {guild_id} AND user_id = {user_id};
                UPDATE users SET vctrack_join_time = UNIX_TIMESTAMP() WHERE guild_id = {guild_id} AND user_id = {user_id}
            ");

            sqlx::raw_sql(&query)
                .execute(database)
                .await
                .unwrap();
        }
    }

    Ok(())
}

// Reset monthly times
pub async fn vctracker_reset_monthly(database: &sqlx::MySqlPool) {
    let current_time = chrono::Utc::now();

    if current_time.format("%d %H:%M:%S").to_string() == "01 00:00:00" { // Day Hour:Minute:Second
        sqlx::query!("UPDATE users SET vctrack_monthly_time = 0")
            .execute(database)
            .await
            .unwrap();

        log::write_log(log::LogType::VCTrackerResetMonthlyComplete);
    }
}