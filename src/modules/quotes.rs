use crate::{Context, Error};
use poise::serenity_prelude as serenity;

//--------------------
// Data
//--------------------
#[derive(sqlx::FromRow)]
struct Quote {
    guild_id: u64,
    adder_id: u64,
    sayer_id: u64,
    quote_id: u32,
    quote: String,
    timestamp: chrono::NaiveDate
}

//--------------------
// Functions
//--------------------
async fn build_single_quote_embed(ctx: Context<'_>, quote: Quote) -> serenity::CreateEmbed {
    // Get user information
    let sayer = serenity::UserId::new(quote.sayer_id).to_user(ctx.http()).await.unwrap();
    let adder = serenity::UserId::new(quote.adder_id).to_user(ctx.http()).await.unwrap();

    // Build embed
    let server_nick = sayer.nick_in(ctx.http(), quote.guild_id).await;
    let sayer_name = if server_nick.is_some() {
        server_nick.unwrap()
    } else if sayer.global_name.is_some() {
        sayer.global_name.as_ref().unwrap().to_string()
    } else {
        String::from(&sayer.name)
    };
    let title = format!("Quote #{} by {}", quote.quote_id, sayer_name);

    let server_nick = adder.nick_in(ctx.http(), quote.guild_id).await;
    let adder_name = if server_nick.is_some() {
        server_nick.unwrap()
    } else if adder.global_name.is_some() {
        adder.global_name.as_ref().unwrap().to_string()
    } else {
        String::from(&adder.name)
    };
    let footer = serenity::CreateEmbedFooter::new(format!("Added by {} on {}", adder_name, quote.timestamp));

    serenity::CreateEmbed::new()
        .colour(0x0b4a6f)
        .description(quote.quote)
        .title(title)
        .thumbnail(sayer.face())
        .footer(footer)
}


//--------------------
// Commands
//--------------------
/// Add quote to the database
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn addquote(
    ctx: Context<'_>,
    sayer: serenity::User,
    #[max_length = 500] quote: String,
) -> Result<(), Error> {
    // Build quote then insert
    let timestamp = chrono::Utc::now().date_naive();
    let mut quote_data = Quote {
        guild_id: ctx.guild_id().unwrap().get(),
        adder_id: ctx.author().id.get(),
        sayer_id: sayer.id.get(),
        quote_id: 0,
        quote,
        timestamp
    };

    let max_quote_id = sqlx::query!("SELECT MAX(quote_id) AS quote_id FROM quotes WHERE guild_id = ?", quote_data.guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .quote_id
        .unwrap_or(0);
    quote_data.quote_id = max_quote_id + 1;

    sqlx::query!("INSERT INTO quotes (guild_id, adder_id, sayer_id, quote_id, quote, timestamp) VALUES (?, ?, ?, ?, ?, ?)",
            quote_data.guild_id,
            quote_data.adder_id,
            quote_data.sayer_id,
            quote_data.quote_id,
            quote_data.quote,
            quote_data.timestamp
        )
        .execute(&ctx.data().database)
        .await
        .unwrap();

    // Build embed then post success
    let quote_embed = build_single_quote_embed(ctx, quote_data).await;

    ctx.send(
        poise::CreateReply::default()
            .content("Quote successfully added!")
            .embed(quote_embed)
    ).await?;

    Ok(())
}

/// Get quote from the database
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn quote(
    ctx: Context<'_>,
    id: Option<u32>,
    user: Option<serenity::User>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Determine query and run it
    let query = if id.is_some() {
        format!("SELECT * FROM quotes WHERE guild_id = {} AND quote_id = {}", guild_id, id.unwrap())
    } else if user.is_some() {
        format!("SELECT * FROM quotes WHERE guild_id = {} AND sayer_id = {} ORDER BY RAND() LIMIT 1", guild_id, user.unwrap().id.get())
    } else {
        format!("SELECT * FROM quotes WHERE guild_id = {} ORDER BY RAND() LIMIT 1", guild_id)
    };
    
    let quote = sqlx::query_as(&query)
        .fetch_one(&ctx.data().database)
        .await;

    let quote = match quote {
        Ok(q) => q,
        Err(_) => return Err("Unable to find that quote in the database!".into())
    };

    // Send quote
    let quote = build_single_quote_embed(ctx, quote).await;
    ctx.send(poise::CreateReply::default().embed(quote)).await?;

    Ok(())
}

/// Delete quote from the database
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn delquote(
    ctx: Context<'_>,
    id: u32
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Check if quote exists first
    let exist = sqlx::query!("SELECT COUNT(quote_id) AS count FROM quotes WHERE guild_id = ? AND quote_id = ?", guild_id, id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .count;

    if exist == 0 {
        return Err("No quote saved with that ID!".into());
    }

    // Delete quote
    let delete_query = sqlx::query!("DELETE FROM quotes WHERE guild_id = ? AND quote_id = ?", ctx.guild_id().unwrap().get(), id)
        .execute(&ctx.data().database)
        .await;

    let _ = match delete_query {
        Ok(_) => ctx.say("Successfully deleted the quote!").await.unwrap(),
        Err(e) => {
            println!("{e}");
            ctx.say("There was an error trying to delete the quote!").await.unwrap()
        }
    };

    Ok(())
}