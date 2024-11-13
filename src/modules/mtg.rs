use crate::{log, Context, Error};
use poise::serenity_prelude as serenity;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

//--------------------
// Data
//--------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    legalities: HashMap<String, String>,

    // Print Fields
    artist: Option<String>,
    games: Vec<String>,
    flavor_text: Option<String>,
    set: String,
    set_name: String,
    image_status: String,
    image_uris: Option<Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
// Embed creation
//--------------------

// Create an embed for a double-faced card
fn create_double_face_embed(scryfall: ScryfallMTGCard) -> Vec<serenity::CreateEmbed> {
    // Create and populate embed vector
    let scryfall_faces = scryfall.card_faces.unwrap();
    let mut card_faces: Vec<serenity::CreateEmbed> = Vec::new();

    for face in scryfall_faces { // For loop needed for indexing into structure
        // Build embed description
        let cost = format!("**Cost:** {}\n", face.mana_cost.replace('{', "(").replace('}', ")"));

        let set_info = format!("**Set:** {} - *{}*\n", scryfall.set.to_uppercase(), scryfall.set_name);

        let available = format!("**Available:** {}\n\n", scryfall.games.join(", "));

        let oracle_text = match &face.oracle_text {
            Some(text) => {
                let mut oracle = text.clone();
                oracle = oracle.replace('{', "(");
                oracle = oracle.replace('}', ")");

                format!("\n{oracle}")
            },
            None => String::new()
        };

        let flavor_text = match &face.flavor_text {
            Some(text) => format!("\n\n{text}"),
            None => String::new()
        };

        let embed_desc = format!("{0}{1}{2}*{3}*\n{4}{5}",
            cost,
            set_info,
            available,
            face.type_line.as_ref().unwrap(),
            oracle_text,
            flavor_text
        );

        // Build and push embed
        let mut card_embed = serenity::CreateEmbed::new()
            .colour(0x000000)
            .title(&face.name)
            .url(scryfall.scryfall_uri.clone())
            .description(embed_desc);

        card_embed = add_fields(&face.power, &face.toughness, &face.loyalty, card_embed);
    
        if scryfall.image_status.as_str() != "missing" {
            let image_url = if scryfall.image_uris.is_none() {
                face.image_uris.as_ref().unwrap()["border_crop"].as_str().unwrap_or("")
            } else {
                scryfall.image_uris.as_ref().unwrap()["border_crop"].as_str().unwrap_or("")
            };

            card_embed = card_embed.image(image_url);
        }

        card_faces.push(card_embed);
    }

    card_faces
}

// Create the embed for a single-face card
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

    card_embed = add_fields(&scryfall.power, &scryfall.toughness, &scryfall.loyalty, card_embed);

    if scryfall.image_status.as_str() != "missing" {
        card_embed = card_embed.image(scryfall.image_uris.unwrap()["border_crop"].as_str().unwrap_or(""));
    }

    card_embed
}

fn create_legalities_embed(scryfall: ScryfallMTGCard) -> serenity::CreateEmbed {
    let mut format_list = String::new();
    for (i, format) in scryfall.legalities.iter().enumerate() {
        format_list = format!("{format_list}**{}:** {}\n",
            beautify_format(format.0.to_string()),
            format.1.replace("not_legal", ":x:").replace("legal", ":white_check_mark:").replace("banned", ":prohibited:").replace("restricted", ":grey_exclamation:")
        );

        if (i + 1) % 11 == 0 {
            format_list = format!("{format_list}\n");
        }
    }

    let legal_desc = format!("**Key:**\n:white_check_mark: - *Legal*\n:x: - *Not Legal*\n:prohibited: - *Banned*\n:grey_exclamation: - *Restricted*\n\n{format_list}");

    serenity::CreateEmbed::new()
        .title(scryfall.name)
        .url(scryfall.scryfall_uri)
        .description(legal_desc)
}

fn add_fields(
    power: &Option<String>,
    toughness: &Option<String>,
    loyalty: &Option<String>,
    mut card_embed: serenity::CreateEmbed
) -> serenity::CreateEmbed {
    if power.is_some() {
        card_embed = card_embed.field("Power:", power.as_ref().unwrap(), true);
    }

    if toughness.is_some() {
        card_embed = card_embed.field("Toughness:", toughness.as_ref().unwrap(), true);
    }

    if loyalty.is_some() {
        card_embed = card_embed.field("Loyalty:", loyalty.as_ref().unwrap(), true);
    }

    card_embed
}

//--------------------
// Function Library
//--------------------

// Format strings in formats
fn beautify_format(mut format: String) -> String {
    format = format.replace("paupercommander", "Pauper Commander");
    format = format.replace("standardbrawl", "Standard Brawl");

    let mut f_chars = format.chars();
    match f_chars.next() {
        Some(c) => c.to_uppercase().chain(f_chars).collect(),
        None => String::new()
    }
}

// Validate parameters in command
fn valid_parameters(
    name: &Option<String>,
    set: &Option<String>,
    collector_num: &Option<i64>
) -> Result<(), Error> {
    if name.is_none() && set.is_none() && collector_num.is_none() {
        return Err("You must include at least the name parameter!".into());
    }

    if set.is_some() && name.is_none() && collector_num.is_none() {
        return Err("You must include the name of the card or the collector number!".into());
    }

    if collector_num.is_some() && set.is_none() {
        return Err("You must include the set code when specifying a collector number!".into());
    }

    Ok(())
}

// Query Scryfall API
async fn scryfall_query(
    client: &reqwest::Client,
    api_url: String
) -> Result<Value, Error> {
    let scryfall = client.get(api_url)
        .send()
        .await;

    let scryfall = match scryfall {
        Ok(data) => data.json::<serde_json::Value>().await.unwrap(),
        Err(err) => {
            log::write_log(log::LogType::MTGScryfallParsingError { error: err.to_string() });
            return Err("There was an error processing your request!".into());
        }
    };

    // Handle errors
    if scryfall["object"].as_str().unwrap() == "error" {
        log::write_log(log::LogType::MTGScryfallError { error: scryfall["details"].as_str().unwrap().to_string() });
        return Err(scryfall["details"].as_str().unwrap().into());
    }

    Ok(scryfall)
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
    #[description = "The full or partial name of the card."] name: Option<String>,
    #[max_length = 3]
    #[description = "The set code for the card"] set: Option<String>,
    #[max_length = 4]
    #[description = "The collector number of the card."] collector_num: Option<i64>,
) -> Result<(), Error> {
    // Validate paramters
    valid_parameters(&name, &set, &collector_num)?;

    let set = set.unwrap_or_default().to_lowercase();

    // Determine API URL and perform query
    let api_url = if collector_num.is_some() {
        format!("https://api.scryfall.com/cards/{}/{}", set, collector_num.unwrap())
    } else {
        format!("https://api.scryfall.com/cards/named?fuzzy={}&set={}", name.unwrap(), set)
    };

    let scryfall = match scryfall_query(&ctx.data().client, api_url).await {
        Ok(v) => v,
        Err(e) => return Err(e)
    };

    // Create Embeds
    let scryfall: ScryfallMTGCard = serde_json::from_value(scryfall).unwrap();

    let card_embed: Vec<serenity::CreateEmbed> = if scryfall.card_faces.is_some() {
        create_double_face_embed(scryfall.clone())
    } else {
        vec![create_single_face_embed(scryfall.clone())]
    };

    let legalities_embed = create_legalities_embed(scryfall.clone());

    // Create button row
    let mut face_index = 0;
    let ctx_id = ctx.id();
    let flip_id = format!("{ctx_id}flip");
    let legalities_id = format!("{ctx_id}legalities");
    let mut legalities_text = "Legalities";
    let mut showing_legalities = false;

    let mut buttons: Vec<serenity::CreateButton> = vec![serenity::CreateButton::new(&legalities_id).label(legalities_text)];

    if scryfall.card_faces.is_some() {
        buttons.push(serenity::CreateButton::new(&flip_id).label("Flip"));
    }

    let buttons = serenity::CreateActionRow::Buttons(buttons);

    // Send embed and handle button events
    ctx.send(poise::CreateReply::default()
        .embed(card_embed[face_index].clone())
        .components(vec![buttons])).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(3600))
        .await
    {
        // Button events
        if press.data.custom_id == legalities_id {
            showing_legalities = !showing_legalities;

            if showing_legalities {
                legalities_text = "Back";
            } else {
                legalities_text= "Legalities";
            }
        } else if press.data.custom_id == flip_id {
            face_index = if face_index == 0 { 1 } else { 0 };

            if showing_legalities {
                showing_legalities = false;
                legalities_text = "Legalities";
            }
        } else {
            continue;
        }

        let mut buttons: Vec<serenity::CreateButton> = vec![serenity::CreateButton::new(&legalities_id).label(legalities_text)];

        if scryfall.card_faces.is_some() {
            buttons.push(serenity::CreateButton::new(&flip_id).label("Flip"));
        }

        let buttons = serenity::CreateActionRow::Buttons(buttons);

        // Update embed
        if showing_legalities {
            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(legalities_embed.clone())
                            .components(vec![buttons])
                    )
                )
                .await?;
        } else {
            press
                .create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(card_embed[face_index].clone())
                            .components(vec![buttons])
                    )
                )
                .await?;
        }
    }
    
    Ok(())
}