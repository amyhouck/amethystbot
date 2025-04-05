use crate::{Context, Error};
use crate::data::user_table_check;
use crate::customgifs::{grab_custom_gifs, GIFType, GIFDBQueryType};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

enum MiscCommand {
    Slap,
    Tea,
    Cake,
    Cookie
}

// Grab random GIF attachment.
async fn grab_misc_gif(
    database: &sqlx::MySqlPool,
    guild_id: u64,
    gif_type: &GIFType
) -> Option<String> {
    let random_gif = grab_custom_gifs(database, gif_type, guild_id, GIFDBQueryType::SingleRandom).await;
    
    if !random_gif.is_empty() {
        let url = random_gif[0].gif_url.to_owned();
        Some(url)
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
    ctx.defer().await?;
    misc_container(ctx, MiscCommand::Slap, victim).await?;

    // // Stat handling
    // let guild_id = ctx.guild_id().unwrap().get();
    // let executioner_id = ctx.author().id.get();
    // user_table_check(ctx, &victim).await;  // - Check victim's existence

    // if ctx.author() != &victim {
    //     let query = format!("UPDATE users SET slap_sent = slap_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
    //         UPDATE users SET slap_received = slap_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

    //     sqlx::raw_sql(&query)
    //         .execute(&ctx.data().database)
    //         .await
    //         .unwrap();
    // }

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
    misc_container(ctx, MiscCommand::Cookie, victim).await?;

    // // Stat handling
    // let guild_id = ctx.guild_id().unwrap().get();
    // let executioner_id = ctx.author().id.get();
    // let victim_id = victim.id.get();
    // user_table_check(ctx, &victim).await;  // - Check victim's existence

    // if ctx.author() != &victim {
    //     let query = format!("UPDATE users SET cookie_sent = cookie_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
    //         UPDATE users SET cookie_received = cookie_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

    //     sqlx::raw_sql(&query)
    //         .execute(&ctx.data().database)
    //         .await
    //         .unwrap();
    // }

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
    misc_container(ctx, MiscCommand::Tea, victim).await?;

    // // Stat handling
    // let guild_id = ctx.guild_id().unwrap().get();
    // let executioner_id = ctx.author().id.get();
    // let victim_id = victim.id.get();
    // user_table_check(ctx, &victim).await;  // - Check victim's existence

    // if ctx.author() != &victim {
    //     let query = format!("UPDATE users SET tea_sent = tea_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
    //         UPDATE users SET tea_received = tea_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}");

    //     sqlx::raw_sql(&query)
    //         .execute(&ctx.data().database)
    //         .await
    //         .unwrap();
    // }

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
    misc_container(ctx, MiscCommand::Cake, victim).await?;

    // // Stat handling
    // let guild_id = ctx.guild_id().unwrap().get();
    // let executioner_id = ctx.author().id.get();
    // let victim_id = victim.id.get();
    // user_table_check(ctx, &victim).await;  // - Check victim's existence

    // // Update stats
    // if ctx.author() != &victim {
    //     let victim_increment = if glados == 9 {
    //         "cake_glados = cake_glados + 1"
    //     } else {
    //         "cake_received = cake_received + 1"
    //     };

    //     let query = format!("UPDATE users SET cake_sent = cake_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
    //         UPDATE users SET {victim_increment} WHERE guild_id = {guild_id} AND user_id = {victim_id}");

    //     sqlx::raw_sql(&query)
    //         .execute(&ctx.data().database)
    //         .await
    //         .unwrap();
    // }

    Ok(())
}

async fn misc_container(
    ctx: Context<'_>,
    command: MiscCommand,
    victim: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // Determine GIF
    let gif_type = match command {
        MiscCommand::Slap => {
            if ctx.author() == &victim {
                GIFType::SlapSelf
            } else {
                GIFType::Slap
            }
        },
        MiscCommand::Cookie => {
            if ctx.author() == &victim {
                GIFType::CookieSelf
            } else {
                GIFType::Cookie
            }
        },
        MiscCommand::Tea => GIFType::Tea,
        MiscCommand::Cake => GIFType::Cake
    };
    
    let mut random_gif = grab_misc_gif(&ctx.data().database, guild_id, &gif_type).await;
    
    // Determine embed message
    let msg = match gif_type {
        GIFType::Tea => {
            if ctx.author() == &victim {
                String::from("You have received some tea!")
            } else {
                format!("{} has given you some tea!", ctx.author())
            }
        },
        GIFType::Slap => format!("{} slaps you around a bit with a large trout!", ctx.author()),
        GIFType::SlapSelf => format!("Stop hitting yourself! Stop hitting yourself!"),
        GIFType::Cake => {
            let mut rng = thread_rng();
            let glados = rng.gen_range(1..=13);
            
            if glados == 9 {
                random_gif = Some(String::from("https://media1.tenor.com/m/I1ZYLNNNEGQAAAAC/portal-glados.gif"));
                String::from("***The cake is a lie***")
            } else {
                format!("{} has given you some cake! Hope you like it!", ctx.author())
            }
        },
        GIFType::Cookie => format!("You have received a cookie from {}!", ctx.author()),
        GIFType::CookieSelf => String::from("NO! No cookies for you!"),
        _ => String::new()
    };
    
    // Build and send embed
    let mut embed = serenity::CreateEmbed::new()
        .description(msg);
        
    let mut reply = poise::CreateReply::default()
        .content(format!("{victim}"));
        
    if let Some(url) = random_gif {
        embed = embed.image(url);
    }
    reply = reply.embed(embed);
    
    ctx.send(reply).await?;
    
    Ok(())
}