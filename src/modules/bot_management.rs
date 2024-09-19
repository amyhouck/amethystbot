use crate::{log, Context, Error};
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    owners_only,
    hide_in_help
)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Shutting down the bot. Goodbye!").await?;
    log::write_log(log::LogType::BotShutdown);
    ctx.framework().shard_manager().shutdown_all().await;

    Ok(())
}

#[poise::command(
    slash_command,
    owners_only,
    hide_in_help
)]
pub async fn reset_display_names(ctx: Context<'_>) -> Result<(), Error> {
    let user_info = sqlx::query!("SELECT guild_id, user_id FROM users")
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();
    let http = ctx.http();

    for user in user_info {
        println!("Updating name for User ID {} - Guild ID {}", user.user_id, user.guild_id);
        let serenity_user = serenity::UserId::new(user.user_id).to_user(http).await.unwrap();
        
        let server_nick = serenity_user.nick_in(http, user.guild_id).await;

        let name = server_nick.unwrap_or_else(||
            match serenity_user.global_name {
                Some(n) => n,
                None => serenity_user.name
            }
        );

        sqlx::query!("UPDATE users SET display_name = ? WHERE guild_id = ? AND user_id = ?", name, user.guild_id, user.user_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        println!("Updated!");
    }

    ctx.say("Complete!").await?;

    Ok(())
}