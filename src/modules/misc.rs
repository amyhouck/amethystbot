use crate::{Context, Error};
use crate::data::user_table_check;
use crate::customgifs::{grab_custom_gifs, GIFType, GIFDBQueryType};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

// Grab random GIF attachment.
async fn grab_misc_gif(
    database: &sqlx::MySqlPool,
    guild_id: u64,
    gif_type: GIFType
) -> Option<serenity::CreateAttachment> {
    let random_gif = grab_custom_gifs(database, &gif_type, guild_id, GIFDBQueryType::SingleRandom).await;
    
    if !random_gif.is_empty() {
        let path = format!("CustomGIFs/{guild_id}/{}/{}.gif", gif_type, random_gif[0].filename);
        Some(serenity::CreateAttachment::path(path).await.unwrap())
    } else {
        None
    }
}

/// Slap slap slap, clap clap clap
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only,
)]
pub async fn slap(
    ctx: Context<'_>,
    #[description = "The user you'd like to slap."] victim: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // All hail RNG
    let gif_type = if &victim == ctx.author() {
        GIFType::SlapSelf
    } else {
        GIFType::Slap
    };
    
    let random_gif = grab_misc_gif(&ctx.data().database, guild_id, gif_type).await;
    
    let embed_msg = if &victim == ctx.author() {
        String::from("Stop hitting yourself...stop hitting yourself!")
    } else {
        format!("{} slaps you around a bit with a large trout!", ctx.author())
    };

    // Switch to using the victim's ID
    let victim_id = victim.id.get();

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .description(embed_msg);
        
    let mut msg = poise::CreateReply::default()
        .content(format!("<@{victim_id}>"));
        
    if let Some(att) = random_gif {
        embed = embed.image(format!("attachment://{}",  att.filename));
        msg = msg.attachment(att);
    }
    msg = msg.embed(embed);

    ctx.send(msg).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    user_table_check(ctx, &victim).await;  // - Check victim's existence

    if ctx.author() != &victim {
        let query = format!("UPDATE users SET slap_sent = slap_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET slap_received = slap_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

        sqlx::raw_sql(&query)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

    Ok(())
}

/// Cookiiiieeeesssssss
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only
)]
pub async fn cookie(
    ctx: Context<'_>,
    #[description = "The user you'd like to cookie."] victim: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let gif_type = if &victim == ctx.author() {
        GIFType::CookieSelf
    } else {
        GIFType::Cookie
    };

    let random_gif = grab_misc_gif(&ctx.data().database, guild_id, gif_type).await;

    let embed_msg = if ctx.author() == &victim {
        String::from("NO! NO COOKIES FOR YOU!")
    } else {
        format!("{} has given you a cookie!", ctx.author())
    };

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .description(embed_msg);
        
    let mut msg = poise::CreateReply::default()
        .content(format!("{victim}"));
        
    if let Some(att) = random_gif {
        embed = embed.image(format!("attachment://{}.gif", att.filename));
        msg = msg.attachment(att);
    }
    msg = msg.embed(embed);

    ctx.send(msg).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(ctx, &victim).await;  // - Check victim's existence

    if ctx.author() != &victim {
        let query = format!("UPDATE users SET cookie_sent = cookie_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET cookie_received = cookie_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

        sqlx::raw_sql(&query)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

    Ok(())
}

/// We love tea
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn tea(
    ctx: Context<'_>,
    #[description = "The user you'd like to tea."] victim: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    let random_gif = grab_misc_gif(&ctx.data().database, guild_id, GIFType::Tea).await;

    let embed_msg = if &victim == ctx.author() {
        String::from("You have received some tea!")
    } else {
        format!("{} has given you some tea!", ctx.author())
    };

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .description(embed_msg);
        
    let mut msg = poise::CreateReply::default()
        .content(format!("{victim}"));
        
    if let Some(att) = random_gif {
        embed = embed.image(format!("attachment://{}.gif", att.filename));
        msg = msg.attachment(att);
    }
    msg = msg.embed(embed);

    ctx.send(msg).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(ctx, &victim).await;  // - Check victim's existence

    if ctx.author() != &victim {
        let query = format!("UPDATE users SET tea_sent = tea_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET tea_received = tea_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

        sqlx::raw_sql(&query)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

    Ok(())
}

/// It is not a lie
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn cake(
    ctx: Context<'_>,
    #[description = "The user you'd like to cake."] victim: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    // Praise the RNG
    let random_gif = grab_misc_gif(&ctx.data().database, guild_id, GIFType::Cake).await;

    let glados = {
        let mut rng = thread_rng();
        rng.gen_range(1..=13)
    };

    // Set message info

    // if glados == 9 {
    //     embed_gif = "https://media1.tenor.com/m/I1ZYLNNNEGQAAAAC/portal-glados.gif";
    // }

    let embed_msg = if glados == 9 {
        String::from("***The cake is a lie***")
    } else {
        format!("{} has given you some cake! Hope you like it!", ctx.author())
    };

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .description(embed_msg);
        
    let mut msg = poise::CreateReply::default()
        .content(format!("{victim}"));
        
    if let Some(att) = random_gif {
        embed = embed.image(format!("attachment://{}.gif", att.filename));
        msg = msg.attachment(att);
    }
    msg = msg.embed(embed);

    ctx.send(msg).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(ctx, &victim).await;  // - Check victim's existence

    // Update stats
    if ctx.author() != &victim {
        let victim_increment = if glados == 9 {
            "cake_glados = cake_glados + 1"
        } else {
            "cake_received = cake_received + 1"
        };

        let query = format!("UPDATE users SET cake_sent = cake_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET {victim_increment} WHERE guild_id = {guild_id} AND user_id = {victim_id}");

        sqlx::raw_sql(&query)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

    Ok(())
}