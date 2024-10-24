use crate::{Context, Error};
use crate::data::user_table_check;
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

/// Slap slap slap, clap clap clap
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only,
)]
pub async fn slap(
    ctx: Context<'_>,
    mut victim: serenity::User
) -> Result<(), Error> {
    // All hail RNG
    let random_gif = {
        let mut rng = thread_rng();

        if &victim == ctx.author() {
            &ctx.data().self_slap_gifs[rng.gen_range(0..ctx.data().self_slap_gifs.len())]
        } else {
            &ctx.data().slap_gifs[rng.gen_range(0..ctx.data().slap_gifs.len())]
        }
    };

    let mut embed_msg = if &victim == ctx.author() {
        String::from("Stop hitting yourself...stop hitting yourself!")
    } else {
        format!("{} slaps you around a bit with a large trout!", ctx.author())
    };

    // Switch to using the victim's ID
    let mut victim_id = victim.id.get();

    let funny = {
        let mut rng = thread_rng();
        rng.gen_range(1..=5)
    };

    if ctx.guild_id().unwrap().get() == 545745915151908865 && funny == 3 {
        victim_id = 811402226643632159;
        embed_msg = format!("{} tried to slap {} but hit <@{}> instead!", ctx.author(), victim, victim_id);
        victim = serenity::UserId::from(victim_id).to_user(ctx.http()).await.unwrap();
    }

    // Build embed
    let embed = serenity::CreateEmbed::new()
        .description(embed_msg)
        .image(random_gif);

    ctx.send(poise::CreateReply::default().content(format!("<@{}>", victim_id)).embed(embed)).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    user_table_check(&ctx.data().database, ctx.http(), ctx.guild_id().unwrap(), &victim).await;  // - Check victim's existence

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
    victim: serenity::User
) -> Result<(), Error> {
    let embed_image = if ctx.author() == &victim {
        "https://media.tenor.com/TJREb7mszbwAAAAd/cat-cookies-cookies.gif"
    } else {
        "https://media.tenor.com/Neg3VGfuntMAAAAC/spongebob-cookies.gif"
    };

    let embed_msg = if ctx.author() == &victim {
        String::from("NO! NO COOKIES FOR YOU!")
    } else {
        format!("{} has given you a cookie!", ctx.author())
    };

    let embed = serenity::CreateEmbed::new()
        .description(embed_msg)
        .image(embed_image);

    ctx.send(poise::CreateReply::default().content(format!("{}", victim)).embed(embed)).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(&ctx.data().database, ctx.http(), ctx.guild_id().unwrap(), &victim).await;  // - Check victim's existence

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
    victim: serenity::User
) -> Result<(), Error> {
    let embed_gif = {
        let mut rng = thread_rng();
        let gif_id = rng.gen_range(0..ctx.data().tea_gifs.len());
        &ctx.data().tea_gifs[gif_id]
    };

    let embed_msg = if &victim == ctx.author() {
        String::from("You have received some tea!")
    } else {
        format!("{} has given you some tea!", ctx.author())
    };

    let embed = serenity::CreateEmbed::new()
        .description(embed_msg)
        .image(embed_gif);

    ctx.send(poise::CreateReply::default().content(format!("{}", victim)).embed(embed)).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(&ctx.data().database, ctx.http(), ctx.guild_id().unwrap(), &victim).await;  // - Check victim's existence

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
    victim: serenity::User
) -> Result<(), Error> {
    // Praise the RNG
    let gif_id = {
        let mut rng = thread_rng();
        rng.gen_range(0..ctx.data().cake_gifs.len())
    };

    let glados = {
        let mut rng = thread_rng();
        rng.gen_range(1..=13)
    };

    // Set message info
    let embed_gif = if glados == 9 {
        "https://media1.tenor.com/m/I1ZYLNNNEGQAAAAC/portal-glados.gif"
    } else {
        &ctx.data().cake_gifs[gif_id]
    };

    let embed_msg = if glados == 9 {
        String::from("***The cake is a lie***")
    } else {
        format!("{} has given you some cake! Hope you like it!", ctx.author())
    };

    let embed = serenity::CreateEmbed::new()
        .description(embed_msg)
        .image(embed_gif);

    ctx.send(poise::CreateReply::default().content(format!("{}", victim)).embed(embed)).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
    user_table_check(&ctx.data().database, ctx.http(), ctx.guild_id().unwrap(), &victim).await;  // - Check victim's existence

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