use crate::{log, Context, Error};

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