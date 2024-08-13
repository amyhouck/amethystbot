use crate::{data, Context, Error};
use crate::data::{user_table_check, User};
use poise::serenity_prelude as serenity;

/// Check user stats
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn stats(
    ctx: Context<'_>,
    user: Option<serenity::User>
) -> Result<(), Error> {
    // Get user id and check database
    let user = match user {
        Some(u) => u,
        None => ctx.author().clone()
    };

    let user_id = user.id;
    let guild_id = ctx.guild_id().unwrap().get();

    // Update Voice Time
    let vc_info = ctx.guild().unwrap().voice_states.clone();
    let vc_info = vc_info.get(&user_id);

    let user_voicechannel = match vc_info {
        Some(i) => i.channel_id,
        None => None
    };
    let user_voicechannel = match user_voicechannel {
        Some(c) => c.get(),
        None => 0
    };

    let ignored_channel = sqlx::query!("SELECT vctrack_ignored_channel FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .vctrack_ignored_channel
        .unwrap_or(0);

    if user_voicechannel != 0 && user_voicechannel != ignored_channel {
        sqlx::query!("UPDATE users SET vctrack_total_time = (UNIX_TIMESTAMP() - vctrack_join_time) + vctrack_total_time WHERE guild_id = ? AND user_id = ?", guild_id, user_id.get())
            .execute(&ctx.data().database)
            .await
            .unwrap();

        sqlx::query!("UPDATE users SET vctrack_join_time = UNIX_TIMESTAMP() WHERE guild_id = ? AND user_id = ?", guild_id, user_id.get())
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }
    
    // Build stats embed
    user_table_check(&ctx.data().database, guild_id, user_id.get()).await;
    let user_data = sqlx::query_as!(User, "SELECT * FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id.get())
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    let server_nick = user.nick_in(ctx.http(), guild_id).await;
    let title_name = if server_nick.is_some() {
        server_nick.unwrap()
    } else if user.global_name.is_some() {
        user.global_name.as_ref().unwrap().to_string()
    } else {
        String::from(&user.name)
    };

    let vctime = format!("{}h {}m {}s",
        (user_data.vctrack_total_time / 60) / 60,
        (user_data.vctrack_total_time / 60) % 60,
        user_data.vctrack_total_time % 60,
    );

    let embed_desc = format!("**Time spent in VC:** {12}\n\n**Cookies sent:** {0}\n**Cookies received:** {1}\n\n**Cakes sent:** {2}\n**Cakes received:** {3}\n**Times GLaDOSed:** {11}\n\n**Cups of tea given:** {6}\n**Cups of tea received:** {7}\n\n**People slapped:** {4}\n**Slaps received:** {5}\n\n**Bombs sent:** {8}\n**Bombs defused:** {9}\n**Times exploded:** {10}",
        user_data.cookie_sent,
        user_data.cookie_received,
        user_data.cake_sent,
        user_data.cake_received,
        user_data.slap_sent,
        user_data.slap_received,
        user_data.tea_sent,
        user_data.tea_received,
        user_data.bomb_sent,
        user_data.bomb_defused,
        user_data.bomb_failed,
        user_data.cake_glados,
        vctime,
    );

    let stat_embed = serenity::CreateEmbed::new()
        .title(format!("{title_name}'s stats"))
        .thumbnail(user.face())
        .description(embed_desc)
        .colour(0x8caac2);

    ctx.send(poise::CreateReply::default().embed(stat_embed)).await?;
    Ok(())
}

/// Get total stats of the server.
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn serverstats(ctx: Context<'_>) -> Result<(), Error> {
    // Query all data from user stats for the server
    let guild_id = ctx.guild_id().unwrap().get();

    let server_data = sqlx::query!("SELECT * FROM users WHERE guild_id = ?", guild_id)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();

    // If no data, return msg. Else combine into server struct
    if server_data.is_empty() {
        ctx.say("This server does not have any stats available yet!").await?;
        return Ok(());
    }

    let mut server_stats = data::ServerStats::default();
    for record in server_data {
        server_stats.bomb_defused += record.bomb_defused;
        server_stats.bomb_failed += record.bomb_failed;
        server_stats.bomb_sent += record.bomb_sent;
        server_stats.cake_sent += record.cake_sent;
        server_stats.cookie_sent += record.cookie_sent;
        server_stats.tea_sent += record.tea_sent;
        server_stats.slap_sent += record.slap_sent;
        server_stats.glados_appearances += record.cake_glados;
    }

    // Build and send stats embed
    let embed_desc = format!("**Cookies sent:** {0}\n**Cakes sent:** {1}\n**Tea sent:** {2}\n**Slaps sent:** {3}\n**GLaDOS appearances:** {7}\n\n**Bombs sent:** {4}\n**Bombs defused:** {5}\n**Bombs exploded:** {6}",
        server_stats.cookie_sent,
        server_stats.cake_sent,
        server_stats.tea_sent,
        server_stats.slap_sent,
        server_stats.bomb_sent,
        server_stats.bomb_defused,
        server_stats.bomb_failed,
        server_stats.glados_appearances,
    );

    let mut embed = serenity::CreateEmbed::new()
        .title("Server Stats")
        .colour(0x8caac2)
        .description(embed_desc);

    if ctx.guild().unwrap().icon_url().is_some() {
        embed = embed.thumbnail(ctx.guild().unwrap().icon_url().unwrap());
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}