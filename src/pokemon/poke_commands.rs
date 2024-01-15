use crate::{Context, Error};
use crate::pokemon::data::{StarterPokemon, Pokemon};
use crate::pokemon::generation::IvEvOptions;
use crate::pokemon::generation::generate_pokemon;

/// Choose a starter pokemon
#[poise::command(
    slash_command
)]
pub async fn starter(
    ctx: Context<'_>,
    #[description = "Starter pokemon choice"] starter: StarterPokemon
) -> Result<(), Error> {
    let new_pokemon: Pokemon = generate_pokemon(&ctx.data().database, starter.to_string(), 5, IvEvOptions::Normal).await;

    ctx.send(|f| f
        .content(format!("Choice: {:?}", new_pokemon))
        .ephemeral(false)
    ).await?;

    Ok(())
}