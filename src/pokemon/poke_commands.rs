use crate::{Context, Error};
use crate::pokemon::data::Pokemon;
use crate::pokemon::generator::IvEvOptions;
use crate::pokemon::generator::generate_pokemon;
use std::collections::HashMap;

const STARTER_CHOICES: [(&str, i64); 29] = [
    // Gen 1
    ("bulbasaur", 1),
    ("charmander", 4),
    ("squirtle", 7),
    // Gen 2
    ("chikorita", 152),
    ("cyndaquil", 155),
    ("totodile", 158),
    // Gen 3
    ("treecko", 252),
    ("torchic", 255),
    ("mudkip", 258),
    // Gen 4
    ("turtwig", 387),
    ("chimchar", 390),
    ("piplup", 393),
    // Gen 5
    ("snivy", 495),
    ("tepig", 498),
    ("oshawott", 501),
    // Gen 6
    ("chespin", 650),
    ("fennekin", 653),
    ("froakie", 656),
    // Gen 7
    ("rowlet", 722),
    ("litten", 725),
    ("popplio", 728),
    // Gen 8
    ("grookey", 810),
    ("scorbunny", 813),
    ("sobble", 816),
    // Gen 9
    ("sprigatito", 906),
    ("fuecoco", 909),
    ("quaxly", 912),
    // Extra
    ("pikachu", 25),
    ("eevee", 133)
];

/// Choose a starter pokemon
#[poise::command(
    slash_command
)]
pub async fn starter(
    ctx: Context<'_>,
    #[description = "Starter pokemon choice (Gens 1-8)"] starter: String,
) -> Result<(), Error> {
    // Validate choice
    let starter_choices: HashMap<&str, i64> = HashMap::from(STARTER_CHOICES);
    let starter = starter_choices.get(starter.to_lowercase().as_str());

    if starter.is_none() {
        return Err("That is not a valid starter choice!".into());
    }

    let new_pokemon: Pokemon = generate_pokemon(&ctx.data().client, *starter.unwrap(), 5, IvEvOptions::Normal).await;

    ctx.send(poise::CreateReply::default()
        .content(format!("You have chosen: {:?}", new_pokemon))
    ).await?;

    Ok(())
}