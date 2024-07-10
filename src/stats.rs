use std::default;

use crate::{Context, Error};
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

    let user_id = user.id.get();
    let guild_id = ctx.guild_id().unwrap().get();
    user_table_check(&ctx.data().database, guild_id, user_id).await;
    let user_data = sqlx::query_as!(User, "SELECT * FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();
    
    // Build stats embed
    let title_name = match user.nick_in(ctx.http(), guild_id).await {
        Some(n) => n,
        None => String::from(&user.name)
    };

    let embed_desc = format!("**Cookies sent:** {0}\n**Cookies received:** {1}\n\n**Cakes sent:** {2}\n**Cakes received:** {3}\n\n**Cups of tea given:** {6}\n**Cups of tea received:** {7}\n\n**People slapped:** {4}\n**Slaps received:** {5}\n\n**Bombs sent:** {8}\n**Bombs defused:** {9}\n**Times exploded:** {10}",
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
        user_data.bomb_failed
    );

    let stat_embed = serenity::CreateEmbed::new()
        .title(format!("{title_name}'s stats"))
        .thumbnail(user.face())
        .description(embed_desc)
        .colour(0x8caac2);

    ctx.send(poise::CreateReply::default().embed(stat_embed)).await?;
    Ok(())
}