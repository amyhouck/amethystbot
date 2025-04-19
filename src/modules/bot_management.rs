use crate::{Context, Error};

/// Set URL for glados easter egg
#[poise::command(
    slash_command,
    owners_only,
    hide_in_help
)]
pub async fn set_glados_gif(
    ctx: Context<'_>,
    #[description = "Glados GIF URL"] url: String
) -> Result<(), Error> {
    sqlx::query!("UPDATE bot_settings SET glados_gif = ?", url)
        .execute(&ctx.data().database)
        .await
        .unwrap();
        
    ctx.say(format!("Updated GLaDOS easter egg GIF to {url}")).await?;
    
    Ok(())
}