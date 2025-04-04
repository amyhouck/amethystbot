use crate::{Context, Error};
use crate::data::user_table_check;
use crate::customgifs::{grab_custom_gifs, GIFType, GIFDBQueryType};
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
    #[description = "The user you'd like to slap."] mut victim: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // All hail RNG
    let gif_type = if &victim == ctx.author() {
        GIFType::SlapSelf
    } else {
        GIFType::Slap
    };

    let slap_gif = grab_custom_gifs(&ctx.data().database, &gif_type, ctx.guild_id().unwrap().get(), GIFDBQueryType::SingleRandom).await;
    let random_gif = if !slap_gif.is_empty() {
        let path = format!("CustomGIFs/{guild_id}/{}/{}.gif", gif_type, slap_gif[0].filename);
        Some(serenity::CreateAttachment::path(path).await?)
    } else {
        None
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
        rng.gen_range(1..=15)
    };

    if ctx.guild_id().unwrap().get() == 545745915151908865 && funny == 3 {
        victim_id = 499008692503707658;
        embed_msg = format!("{} tried to slap {} but hit <@{}> instead!", ctx.author(), victim, victim_id);
        victim = serenity::UserId::from(victim_id).to_user(ctx.http()).await.unwrap();
    }

    // Build embed
    let mut embed = serenity::CreateEmbed::new()
        .description(embed_msg);
        
    let mut msg = poise::CreateReply::default()
        .content(format!("<@{victim_id}>"));
        
    if let Some(att) = random_gif {
        embed = embed.image(format!("attachment://{}.gif", slap_gif[0].filename));
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

    let cookie_gif = grab_custom_gifs(&ctx.data().database, &gif_type, ctx.guild_id().unwrap().get(), GIFDBQueryType::SingleRandom).await;

    let random_gif = if !cookie_gif.is_empty() {
        let path = format!("CustomGIFs/{guild_id}/{}/{}.gif", gif_type, cookie_gif[0].filename);
        Some(serenity::CreateAttachment::path(path).await?)
    } else {
        None
    };

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
        embed = embed.image(format!("attachment://{}.gif", cookie_gif[0].filename));
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
    let tea_gif = grab_custom_gifs(&ctx.data().database, &GIFType::Tea, ctx.guild_id().unwrap().get(), GIFDBQueryType::SingleRandom).await;

    let random_gif = if !tea_gif.is_empty() {
        let path = format!("CustomGIFs/{guild_id}/{}/{}.gif", GIFType::Tea, tea_gif[0].filename);
        Some(serenity::CreateAttachment::path(path).await?)
    } else {
        None
    };

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
        embed = embed.image(format!("attachment://{}.gif", tea_gif[0].filename));
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
    let cake_gif = grab_custom_gifs(&ctx.data().database, &GIFType::Cake, ctx.guild_id().unwrap().get(), GIFDBQueryType::SingleRandom).await;

    let glados = {
        let mut rng = thread_rng();
        rng.gen_range(1..=13)
    };

    // Set message info
    let random_gif = if !cake_gif.is_empty() {
        let path = format!("CustomGIFs/{guild_id}/{}/{}.gif", GIFType::Cake, cake_gif[0].filename);
        Some(serenity::CreateAttachment::path(path).await?)
    } else {
        None
    };

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
        embed = embed.image(format!("attachment://{}.gif", cake_gif[0].filename));
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