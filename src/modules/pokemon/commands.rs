use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use crate::pokemon::data;

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

    Ok(())
}