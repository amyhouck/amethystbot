use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use crate::pokemon::data;
use crate::pokemon::pokemon::generate_new_pokemon;

//--------------------
// Command Data
//--------------------
#[derive(poise::ChoiceParameter, Debug)]
enum StarterGroupA {
    Bulbasaur = 1,
    Charmander = 4,
    Squirtle = 7
}

//--------------------
// Commands
//--------------------
#[poise::command(
    slash_command,
    guild_only,
    subcommands("starter")
)]
pub async fn pokemon(
    _: Context<'_>
) -> Result<(), Error> {
    Ok(())
}

/// Choose a starter pokemon
#[poise::command(
    slash_command,
    guild_only,
    //ephemeral
)]
pub async fn starter(
    ctx: Context<'_>,
    group_a: Option<StarterGroupA>,
) -> Result<(), Error> {
    // Validate options
    if group_a.is_none() {
        return Err("You must choose one pokemon!".into());
    }

    let starter_id = group_a.unwrap() as u32;

    // Handle Pokemon generation
    let pokemon_base_info = sqlx::query_as!(data::PokemonBase, "SELECT * FROM pokemon_base_info WHERE id = ?", starter_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    let pokemon = generate_new_pokemon(pokemon_base_info);

    ctx.say(format!("{:?}", pokemon)).await?;

    Ok(())
}