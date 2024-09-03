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
    sayer: serenity::User,
    #[max_length = 500] quote: String,
    date: Option<String>
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
    member_cooldown = 5,
    check = "quote_role_check",
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

/// Set/unset a role required to modify quotes
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn setquoterole(
    ctx: Context<'_>,
    role: Option<serenity::Role>
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