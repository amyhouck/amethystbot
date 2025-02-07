use crate::{data::{determine_display_username, user_table_check}, Context, Error};
use futures::future;
use poise::serenity_prelude as serenity;

//--------------------
// Data
//--------------------
#[derive(Default, sqlx::FromRow)]
struct Quote {
    guild_id: u64,
    adder_id: u64,
    sayer_id: u64,
    quote_id: u32,
    quote: String,
    timestamp: chrono::NaiveDate,
    sayer_display_name: String,
    adder_display_name: String,
}

//--------------------
// Functions
//--------------------
async fn build_single_quote_embed(http: &serenity::Http, quote: Quote) -> serenity::CreateEmbed {
    // Get serenity user
    let sayer = serenity::UserId::new(quote.sayer_id).to_user(http).await.unwrap();

    // Build embed
    let title = format!("Quote #{} by {}", quote.quote_id, quote.sayer_display_name);

    let footer = serenity::CreateEmbedFooter::new(format!("Added by {} on {}", quote.adder_display_name, quote.timestamp));

    serenity::CreateEmbed::new()
        .colour(0x0b4a6f)
        .description(quote.quote)
        .title(title)
        .thumbnail(sayer.face())
        .footer(footer)
}

// Check if the quote command requires a role and determine if the command can be used
async fn quote_role_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let role_id = sqlx::query!("SELECT quotes_required_role FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .quotes_required_role;

    if role_id.is_none() {
        return Ok(true);
    }

    let role_id = serenity::RoleId::new(role_id.unwrap());
    let guild_roles = ctx.guild_id().unwrap().roles(&ctx.http()).await.unwrap();
    let role = match guild_roles.get(&role_id) {
        Some(r) => r,
        None => return Err("A role is set to be required for this command, but it doesn't exist!".into())
    };

    if !ctx.author().has_role(ctx.http(), guild_id, role_id).await.unwrap() {
        ctx.send(
    poise::CreateReply::default()
                .content(format!("You must have the '{}' role to run this command!", role.name))
                .ephemeral(true)
        ).await?;

        return Ok(false);
    }

    Ok(true)
}

fn split_quotes_into_pages(guild_quotes: Vec<Quote>) -> Vec<String> {
    let mut page_content = String::new();
    let mut pages: Vec<String> = Vec::new();
    for (i, quote) in guild_quotes.iter().enumerate() {
        page_content = format!("{page_content}**{}.** {} \n*\\- {} {}* (ID: {})\n\n",
            i + 1,
            quote.quote,
            quote.sayer_display_name,
            quote.timestamp,
            quote.quote_id,
        );

        // Split content into more pages as necessary
        if i + 1 == guild_quotes.len() {
            pages.push(page_content);
            break;
        }

        if (i + 1) % 5 == 0 {
            pages.push(page_content);
            page_content = String::new();
        }
    }

    pages
}


//--------------------
// Commands
//--------------------
/// Add quote to the database
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
    check = "quote_role_check",
)]
pub async fn addquote(
    ctx: Context<'_>,
    #[description = "The person who said the quote."] sayer: serenity::User,
    #[max_length = 500]
    #[description = "The quote to record."] quote: String,
    #[description = "Optionally add a date for the quote."] date: Option<String>
) -> Result<(), Error> {
    // Build quote then insert
    let timestamp = if date.is_some() {
        match chrono::NaiveDate::parse_from_str(&date.unwrap(), "%F") {
            Ok(d) => d,
            Err(_) => return Err("You must formt the date with YYYY-MM-DD!".into())
        }
    } else {
        chrono::Utc::now().date_naive()
    };

    let mut quote_data = Quote {
        guild_id: ctx.guild_id().unwrap().get(),
        adder_id: ctx.author().id.get(),
        sayer_id: sayer.id.get(),
        quote,
        timestamp,
        sayer_display_name: determine_display_username(ctx.http(), &sayer, ctx.guild_id().unwrap()).await,
        adder_display_name: determine_display_username(ctx.http(), ctx.author(), ctx.guild_id().unwrap()).await,
        ..Default::default()
    };

    let max_quote_id = sqlx::query!("SELECT MAX(quote_id) AS quote_id FROM quotes WHERE guild_id = ?", quote_data.guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .quote_id
        .unwrap_or(0);
    quote_data.quote_id = max_quote_id + 1;

    let insert_query = sqlx::query!("INSERT INTO quotes (guild_id, adder_id, sayer_id, quote_id, quote, timestamp, adder_display_name, sayer_display_name) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            quote_data.guild_id,
            quote_data.adder_id,
            quote_data.sayer_id,
            quote_data.quote_id,
            quote_data.quote,
            quote_data.timestamp,
            quote_data.adder_display_name,
            quote_data.sayer_display_name
        )
        .execute(&ctx.data().database);

    let sayer_check = user_table_check(ctx, &sayer);

    let _ = future::join(insert_query, sayer_check).await;

    // Build embed then post success
    let quote_embed = build_single_quote_embed(ctx.http(), quote_data).await;

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
    #[description = "Search for a specific quote by ID."] id: Option<u32>,
    #[description = "Grab a random quote said by a user."] user: Option<serenity::User>,
    #[description = "Grab a quote that contains the given text."] text: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Determine query and run it
    let query = if id.is_some() {
        format!("SELECT * FROM quotes WHERE guild_id = {} AND quote_id = {}", guild_id, id.unwrap())
    } else if user.is_some() {
        format!("SELECT * FROM quotes WHERE guild_id = {} AND sayer_id = {} ORDER BY RAND() LIMIT 1", guild_id, user.unwrap().id.get())
    } else if text.is_some() {
        format!("SELECT * FROM quotes WHERE guild_id = {} AND quote LIKE \"%{}%\"", guild_id, text.unwrap())
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
    let quote = build_single_quote_embed(ctx.http(), quote).await;
    ctx.send(poise::CreateReply::default().embed(quote)).await?;

    Ok(())
}

/// Delete quote from the database
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
    check = "quote_role_check",
)]
pub async fn delquote(
    ctx: Context<'_>,
    #[description = "The ID of the quote to delete."] id: u32
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Check if quote exists first
    let quote = sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE guild_id = ? AND quote_id = ?", guild_id, id)
        .fetch_optional(&ctx.data().database)
        .await
        .unwrap();

    let _ = match quote {
        Some(q) => q,
        None => return Err("No quote saved with that ID!".into())
    };

    // Delete quote
    let delete_query = sqlx::query!("DELETE FROM quotes WHERE guild_id = ? AND quote_id = ?", guild_id, id)
        .execute(&ctx.data().database)
        .await;

    let _ = match delete_query {
        Ok(_) => ctx.say("Successfully deleted the quote!").await.unwrap(),
        Err(e) => {
            println!("{e}");
            ctx.say("There was an error trying to delete the quote!").await.unwrap()
        }
    };

    // Adjust quote IDs
    sqlx::query!("UPDATE quotes SET quote_id = quote_id - 1 WHERE guild_id = ? AND quote_id > ?", guild_id, id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    Ok(())
}

/// Set/unset a role required to modify quotes
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn setquoterole(
    ctx: Context<'_>,
    #[description = "The role required to add or delete quotes."] role: Option<serenity::Role>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    if role.is_none() { // Remove role requirement
        sqlx::query!("UPDATE guild_settings SET quotes_required_role = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say("No longer requiring a role to modify quotes!").await?;
    } else { // Add/modify role requirement
        let role_id = role.as_ref().unwrap().id.get();
        let role_name = role.unwrap().name;

        sqlx::query!("UPDATE guild_settings SET quotes_required_role = ? WHERE guild_id = ?", role_id, guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say(format!("Now requiring the {role_name} role to modify quotes!")).await?;
    }

    Ok(())
}

/// List quotes in a server. Up to 10 on each page.
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn listquotes(
    ctx: Context<'_>,
    #[description = "Pull all quotes from a user."] user: Option<serenity::User>
) -> Result<(), Error> {
    // Grab sorted guild quotes into vector
    let guild_id = ctx.guild_id().unwrap().get();
    
    let user_id = match user {
        Some(u) => u.id.get(),
        None => 0
    };

    let guild_quotes: Vec<Quote> = if user_id == 0 {
        sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE guild_id = ? ORDER BY quote_id", guild_id)
            .fetch_all(&ctx.data().database)
            .await
            .unwrap()
    } else {
        sqlx::query_as!(Quote, "SELECT * FROM quotes WHERE guild_id = ? AND sayer_id = ? ORDER BY quote_id", guild_id, user_id)
            .fetch_all(&ctx.data().database)
            .await
            .unwrap()
    };

    if guild_quotes.len() == 0 {
        return Err("No quotes found!".into());
    }

    // Create initial embed
    let pages = split_quotes_into_pages(guild_quotes);
    let mut page_num = 0;
    let ctx_id = ctx.id();
    let prev_id = format!("{ctx_id}prev");
    let next_id = format!("{ctx_id}next");

    let buttons: Vec<serenity::CreateButton> = vec![
        serenity::CreateButton::new(&prev_id).label("Previous"),
        serenity::CreateButton::new(&next_id).label("Next")
    ];
    let buttons = serenity::CreateActionRow::Buttons(buttons);

    let embed = serenity::CreateEmbed::new()
        .description(&pages[page_num])
        .colour(0x0b4a6f)
        .title("Quotes");

    ctx.send(poise::CreateReply::default()
        .embed(embed)
        .components(vec![buttons])
    ).await?;

    // Handle button interactions
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(1800))
        .await {
            if press.data.custom_id == prev_id {
                page_num = page_num.checked_sub(1).unwrap_or(pages.len() - 1)
            } else if press.data.custom_id == next_id {
                page_num += 1;
                if page_num >= pages.len() { page_num = 0; }
            } else {
                continue;
            }

            press.create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(serenity::CreateEmbed::new()
                            .description(&pages[page_num])
                            .colour(0x0b4a6f)
                            .title("Quotes")
                        )
                )
            ).await?;
    }

    Ok(())
}