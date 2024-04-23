use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use serde::{Serialize, Deserialize};

//--------------------
// Data Structures
//--------------------
#[derive(Clone, Debug, Serialize, Deserialize)]
struct DigimonTCGCard {
    name: String,
    r#type: String,
    color: String,
    stage: Option<String>,
    digi_type: Option<String>,
    attribute: Option<String>,
    level: Option<i64>,
    play_cost: Option<i64>,
    evolution_cost: Option<i64>,
    cardrarity: Option<String>,
    artist: Option<String>,
    dp: Option<i64>,
    cardnumber: String,
    maineffect: Option<String>,
    soureeffect: Option<String>,
    set_name: String,
    card_sets: Vec<String>,
    image_url: String,
}

//--------------------
// Function library
//--------------------
// Request digimon card data
async fn get_digicard_data(url: String, client: &reqwest::Client) -> Result<Vec<DigimonTCGCard>, Error> {
    // Begin request
    let digimon_request = client.get(url)
        .send()
        .await;

    let digimon_request = match digimon_request {
        Ok(data) => data.json::<serde_json::Value>().await.unwrap(),
        Err(e) => {
            println!("[ ERROR - Digimon ] {e}");
            return Err("An error occurred processing your request!".into());
        }
    };

    // Convert and return data
    if digimon_request["error"].is_string() {
        return Err(digimon_request["error"].as_str().unwrap().into());
    }

    let digidata: Vec<DigimonTCGCard> = serde_json::from_value(digimon_request).unwrap();

    Ok(digidata)
}

// Build card embed vector
fn build_card_embed_vector(digidata: Vec<DigimonTCGCard>) -> Vec<serenity::CreateEmbed> {
    let mut card_embeds: Vec<serenity::CreateEmbed> = Vec::new();

    // Build embeds then push
    for (i, card) in digidata.iter().enumerate() {
        let mut card_embed = serenity::CreateEmbed::new()
            .colour(0xFFFFFF)
            .title(format!("**{}. {}**", i + 1, card.name))
            .image(&card.image_url);

        // Build Description
        let mut embed_desc = format!("**Type:** {}", card.r#type);
        let mut card_traits = String::new();

        if card.level.is_some() {
            embed_desc = format!("{embed_desc}\n**Level:** {}", card.level.unwrap());
        }

        if card.dp.is_some() {
            embed_desc = format!("{embed_desc}\n**DP:** {}", card.dp.unwrap());
        }

        if card.play_cost.is_some() {
            embed_desc = format!("{embed_desc}\n**Play Cost:** {}", card.play_cost.unwrap());
        }

        if card.evolution_cost.is_some() {
            embed_desc = format!("{embed_desc}\n**Digivolution Cost:** {}", card.evolution_cost.unwrap());
        }

        /*if card.digi_type.is_some() && card.attribute.is_some() && card.stage.is_some() {
            embed_desc = format!("{embed_desc}\n**Traits:** *{}* | *{}* | *{}*",
                card.stage.as_ref().unwrap(),
                card.attribute.as_ref().unwrap(),
                card.digi_type.as_ref().unwrap());
        }*/

        if card.stage.is_some() {
            card_traits = format!("{} |", card.stage.as_ref().unwrap());
        }

        if card.attribute.is_some() {
            card_traits = format!("{card_traits} {} |", card.attribute.as_ref().unwrap());
        }

        if card.digi_type.is_some() {
            card_traits = format!("{card_traits} {}", card.digi_type.as_ref().unwrap());
        }

        if !card_traits.is_empty() {
            embed_desc = format!("{embed_desc}\n**Traits:** {card_traits}");
        }

        if card.maineffect.is_some() {
            embed_desc = format!("{embed_desc}\n\n**Main Effect:** \n{}", card.maineffect.as_ref().unwrap());
        }

        if card.soureeffect.is_some() {
            embed_desc = format!("{embed_desc}\n\n**Source Effect:** \n{}", card.soureeffect.as_ref().unwrap());
        }

        card_embed = card_embed.description(embed_desc);

        card_embeds.push(card_embed);
    }

    card_embeds
}

//--------------------
// Commands
//--------------------
#[poise::command(
    slash_command,
    member_cooldown = 1,
    subcommands("search")
)]
pub async fn digimon(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn search(
    ctx: Context<'_>,
    name: Option<String>
) -> Result<(), Error> {
    ctx.defer().await?;

    // Build API URL
    let mut api_url = String::from("https://digimoncard.io/api-public/search.php?series=Digimon Card Game&");

    if name.is_some() {
        api_url = format!("{api_url}n={}&", name.unwrap());
    }

    // Request data and build paginated embed
    let digidata = match get_digicard_data(api_url, &ctx.data().client).await {
        Ok(ok) => ok,
        Err(e) => return Err(e)
    };

    let digidata = build_card_embed_vector(digidata);

    if !digidata.is_empty() {
        ctx.send(poise::CreateReply::default()
            .embed(digidata[0].clone())).await?;
    }

    Ok(())
}