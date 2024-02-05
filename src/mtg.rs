use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serde::{Serialize, Deserialize};
use serde_json::Value;

//--------------------
// Data
//--------------------
#[derive(Debug, Serialize, Deserialize)]
struct MTGCard {
    // Core fields
    scryfall_uri: String,

    // Gameplay fields
    name: String,
    cmc: f64,
    color_identity: Vec<char>,
    mana_cost: Option<String>,
    loyalty: Option<String>,
    power: Option<String>,
    toughness: Option<String>,
    oracle_text: Option<String>,
    type_line: String,
    reserved: bool,

    // Print Fields
    artist: Option<String>,
    games: Vec<String>,
    flavor_text: Option<String>,
    set: String,
    set_name: String,
    image_status: String,
    image_uris: Value,
}

//--------------------
// Function library
//--------------------

//--------------------
// Commands
//--------------------
#[poise::command(
    slash_command,
    member_cooldown = 1,
    subcommands("card")
)]
pub async fn mtg(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Search for a specific MTG card
#[poise::command(slash_command)]
pub async fn card(
    ctx: Context<'_>,
    name: String,
    #[max_length = 3] set: Option<String>,
) -> Result<(), Error> {
    // Run a fuzzy search for the card
    let scryfall = ctx.data().client.get(format!("https://api.scryfall.com/cards/named?fuzzy={}&set={}", name, set.as_ref().unwrap_or(&String::new())))
        .send()
        .await;

    let scryfall = match scryfall {
        Ok(data) => data.json::<serde_json::Value>().await.unwrap(),
        Err(err) => {
            println!("[ Scryfall - ERROR ] Encountered error requesting from Scryfall: {err}");
            return Err("There was an error processing your request!".into());
        }
    };

    // Handle errors
    if scryfall["object"].as_str().unwrap() == "error" {
        println!("[ Scryfall - ERROR ] Encountered error for Scryfall data ({}) - {}", scryfall["status"].as_i64().unwrap(), scryfall["code"].as_str().unwrap());

        let set_error = if set.is_some() {
            format!(" in the set \"{}\"", set.unwrap())
        } else {
            String::new()
        };

        return Err(format!("{}{}", scryfall["details"].as_str().unwrap(), set_error).into());
    }

    // * Create Embed
    let scryfall: MTGCard = serde_json::from_value(scryfall).unwrap();
    println!("{:?}", scryfall);

    // Build Embed Description
    let cost = match scryfall.mana_cost {
        Some(mut mana_cost) => {
            mana_cost = mana_cost.replace('{', "(");
            mana_cost = mana_cost.replace('}', ")");

            if mana_cost.is_empty() { mana_cost = "None".to_string(); }

            format!("**Cost:** {mana_cost}\n")
        },
        None => String::new()
    };

    let set_info = format!("**Set:** {} - *{}*\n", scryfall.set.to_uppercase(), scryfall.set_name);

    let available = format!("**Available:** {}\n\n", scryfall.games.join(", "));

    let oracle_text = match scryfall.oracle_text {
        Some(mut text) => {
            text = text.replace('{', "(");
            text = text.replace('}', ")");

            format!("\n{text}")
        },
        None => String::new()
    };

    let flavor_text = match scryfall.flavor_text {
        Some(text) => format!("\n\n{text}"),
        None => String::new()
    };

    let embed_desc = format!("{0}{1}{2}*{3}*\n{4}{5}",
        cost,
        set_info,
        available,
        scryfall.type_line,
        oracle_text,
        flavor_text
    );

    // Embed
    let mut card_embed = serenity::CreateEmbed::new()
        .colour(0x000000)
        .title(scryfall.name)
        .url(scryfall.scryfall_uri)
        .description(embed_desc);

    // Alter embedd based on available information
    if scryfall.power.is_some() {
        card_embed = card_embed.field("Power:", scryfall.power.unwrap(), true);
    }

    if scryfall.toughness.is_some() {
        card_embed = card_embed.field("Toughness:", scryfall.toughness.unwrap(), true);
    }

    if scryfall.loyalty.is_some() {
        card_embed = card_embed.field("Loyalty:", scryfall.loyalty.unwrap(), true);
    }

    if scryfall.image_status.as_str() != "missing" {
        card_embed = card_embed.image(scryfall.image_uris["border_crop"].as_str().unwrap());
    }

    ctx.send(poise::CreateReply::default().embed(card_embed)).await?;
    Ok(())
}