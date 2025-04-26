use crate::{Context, Error};

// Bot Gif Structure
#[derive(poise::ChoiceParameter)]
enum BotGif {
    Glados,
    RouletteClick,
    RouletteFire
}

impl std::fmt::Display for BotGif {
   fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
       match self {
           BotGif::Glados => write!(f, "glados"),
           BotGif::RouletteClick => write!(f, "roulette_click"),
           BotGif::RouletteFire => write!(f, "roulette_fire")
       }
   }
}

/// Set URL for determined GIFs
#[poise::command(
    slash_command,
    owners_only,
    hide_in_help
)]
pub async fn set_bot_gif(
    ctx: Context<'_>,
    #[description = "The type of bot GIF to set."] bot_gif: BotGif,
    #[description = "GIF Url"] url: String,
) -> Result<(), Error> {
   let query = format!("UPDATE bot_settings SET {bot_gif}_gif = {url}");

   sqlx::raw_sql(&query)
       .execute(&ctx.data().database)
       .await
       .unwrap();

    ctx.say(format!("Updated {bot_gif}_gif with {url}")).await?;

    Ok(())
}
