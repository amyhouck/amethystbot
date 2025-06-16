use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tracing::{info, warn};

/// Settings for the VC time tracker
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    subcommands("ignorechannel"),
    member_cooldown = 5,
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
    guild_only,
    member_cooldown = 5,
    ephemeral
)]
pub async fn vctop(
    ctx: Context<'_>,
) -> Result<(), Error> {
    // Grab guild info and recheck voice times
    let guild_id = ctx.guild_id().unwrap().get();

    let ctx_voice_states = ctx.guild().unwrap().voice_states.clone();
    let mut voice_states: Vec<serenity::VoiceState> = Vec::new();

    for (_, v) in ctx_voice_states {
        voice_states.push(v);
    }
    
    let futures = voice_states.iter().map(|vs| recheck_time(vs, &ctx.data().database));
    futures::future::join_all(futures).await;

    // Grab information
    let vctop_all = sqlx::query!("SELECT display_name, vctrack_total_time FROM users WHERE guild_id = ? ORDER BY vctrack_total_time DESC LIMIT 10", guild_id)
        .fetch_all(&ctx.data().database);
    let vctop_monthly = sqlx::query!("SELECT display_name, vctrack_monthly_time FROM users WHERE guild_id = ? ORDER BY vctrack_monthly_time DESC LIMIT 10", guild_id)
        .fetch_all(&ctx.data().database);
        
    let futures_data = futures::future::join(vctop_all, vctop_monthly).await;
    
    let vctop_all: Vec<(String, u32)> = futures_data.0
        .unwrap()
        .iter()
        .map(|r| (r.display_name.to_string(), r.vctrack_total_time))
        .collect();
    let vctop_monthly: Vec<(String, u32)> = futures_data.1
        .unwrap()
        .iter()
        .map(|r| (r.display_name.to_string(), r.vctrack_monthly_time))
        .collect();

    // Build embeds and do interaction
    let vctop_pages: Vec<serenity::CreateEmbed> = vec![
        build_vctop_embed("VCTop All-Time Leaderboard", vctop_all),
        build_vctop_embed("VCTop Monthly Leaderboard", vctop_monthly)
    ];
    let mut vctop_page = 0usize;
    
    let ctx_id = ctx.id();
    let alltime_id = format!("{ctx_id}alltime");
    let monthly_id = format!("{ctx_id}monthly");
    
    let buttons: Vec<serenity::CreateButton> = vec![
        serenity::CreateButton::new(&alltime_id).label("All-Time"),
        serenity::CreateButton::new(&monthly_id).label("Monthly")
    ];
    let buttons = serenity::CreateActionRow::Buttons(buttons);
    
    ctx.send(poise::CreateReply::default()
        .embed(vctop_pages[vctop_page].clone())
        .components(vec![buttons])
    ).await?;
    
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(600))
        .await
    {
        if press.data.custom_id == alltime_id {
            vctop_page = 0;
        } else if press.data.custom_id == monthly_id {
            vctop_page = 1;
        } else {
            continue;
        }
        
        press.create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(vctop_pages[vctop_page].clone())
            )
        ).await?;
    }

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
            warn!("[ VCTRACKER ] SAFEGUARD - Skipping user's time update. Guild ID: {guild_id} - User ID: {user_id}");
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

        info!("[ VCTRACKER ] Reset monthly VC times for every user.");
    }
}

// Build vctop embeds
fn build_vctop_embed(
    title: &str,
    vctime_record: Vec<(String, u32)>,
) -> serenity::CreateEmbed {
    let mut embed_desc = String::new();
    for (i, user) in vctime_record.iter().enumerate() {
        // Format into embed_desc
        embed_desc = format!("{embed_desc}**{}.** {} - {}h {}m {}s\n",
            i + 1,
            user.0,
            (user.1 / 60) / 60,
            (user.1 / 60) % 60,
            user.1 % 60,
        );
    }
    
    serenity::CreateEmbed::new()
        .title(title)
        .colour(0xcc3842)
        .description(embed_desc)
}
