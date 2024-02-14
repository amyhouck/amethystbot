use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serde::{Serialize, Deserialize};
use serde_json::Value;

//--------------------
// Data
//--------------------
#[derive(Debug, Serialize, Deserialize)]
struct ScryfallMTGCard {
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
    card_faces: Option<Vec<ScryfallMTGCardFace>>,

    // Print Fields
    artist: Option<String>,
    games: Vec<String>,
    flavor_text: Option<String>,
    set: String,
    set_name: String,
    image_status: String,
    image_uris: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ScryfallMTGCardFace {
    name: String,
    mana_cost: String,
    cmc: Option<f64>,
    power: Option<String>,
    toughness: Option<String>,
    loyalty: Option<String>,
    flavor_text: Option<String>,
    oracle_text: Option<String>,
    image_uris: Option<Value>,
    type_line: Option<String>,
}

//--------------------
// Function library
//--------------------

fn create_double_face_embed(scryfall: ScryfallMTGCard) -> Vec<serenity::CreateEmbed> {
    // Create and populate embed vector
    let scryfall_faces = scryfall.card_faces.unwrap();
    let mut card_faces: Vec<serenity::CreateEmbed> = Vec::new();

    for i in 0..=1 { // For loop needed for indexing into structure
        // Build embed description
        let cost = format!("**Cost:** {}\n", scryfall_faces[i].mana_cost.replace('{', "(").replace('}', ")"));

        let set_info = format!("**Set:** {} - *{}*\n", scryfall.set.to_uppercase(), scryfall.set_name);

        let available = format!("**Available:** {}\n\n", scryfall.games.join(", "));

        let oracle_text = match &scryfall_faces[i].oracle_text {
            Some(text) => {
                let mut oracle = text.clone();
                oracle = oracle.replace('{', "(");
                oracle = oracle.replace('}', ")");

                format!("\n{oracle}")
            },
            None => String::new()
        };

        let flavor_text = match &scryfall_faces[i].flavor_text {
            Some(text) => format!("\n\n{text}"),
            None => String::new()
        };

        let embed_desc = format!("{0}{1}{2}*{3}*\n{4}{5}",
            cost,
            set_info,
            available,
            scryfall_faces[i].type_line.as_ref().unwrap(),
            oracle_text,
            flavor_text
        );

        // Build and push embed
        let mut card_embed = serenity::CreateEmbed::new()
            .colour(0x000000)
            .title(&scryfall_faces[i].name)
            .url(scryfall.scryfall_uri.clone())
            .description(embed_desc);

        if scryfall_faces[i].power.is_some() {
            card_embed = card_embed.field("Power:", scryfall_faces[i].power.as_ref().unwrap(), true);
        }
    
        if scryfall_faces[i].toughness.is_some() {
            card_embed = card_embed.field("Toughness:", scryfall_faces[i].toughness.as_ref().unwrap(), true);
        }

        if scryfall_faces[i].loyalty.is_some() {
            card_embed = card_embed.field("Loyalty:", scryfall_faces[i].loyalty.as_ref().unwrap(), true);
        }
    
        if scryfall.image_status.as_str() != "missing" {
            let image_url = if scryfall.image_uris.is_none() {
                scryfall_faces[i].image_uris.as_ref().unwrap()["border_crop"].as_str().unwrap_or("")
            } else {
                scryfall.image_uris.as_ref().unwrap()["border_crop"].as_str().unwrap_or("")
            };

            card_embed = card_embed.image(image_url);
        }

        card_faces.push(card_embed);
    }

    card_faces
}

fn create_single_face_embed(scryfall: ScryfallMTGCard) -> serenity::CreateEmbed {
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
        card_embed = card_embed.image(scryfall.image_uris.unwrap()["border_crop"].as_str().unwrap_or(""));
    }

    card_embed
}

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

    // Create Embed
    let scryfall: ScryfallMTGCard = serde_json::from_value(scryfall).unwrap();

    if scryfall.card_faces.is_some() { // * Double faced cards
        let card_faces = create_double_face_embed(scryfall);

        // Create buttoned embed
        let mut face_index = 0;
        let ctx_id = ctx.id();
        let flip_id = format!("{ctx_id}flip");

        let button = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&flip_id).label("Flip"),
        ]);
        
        ctx.send(poise::CreateReply::default()
            .embed(card_faces[face_index].clone())
            .components(vec![button])).await?;

        while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
            .timeout(std::time::Duration::from_secs(3600))
            .await
        {
            if press.data.custom_id == flip_id {
                face_index = if face_index == 0 { 1 } else { 0 };
            } else {
                continue;
            }

            press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(card_faces[face_index].clone())
                )
            )
            .await?;
        }
    } else { // * Single faced cards
        let card_embed = create_single_face_embed(scryfall);
        ctx.send(poise::CreateReply::default().embed(card_embed)).await?;
    }

    
    Ok(())
}