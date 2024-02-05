use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

/// Slap slap slap, clap clap clap
#[poise::command(
    slash_command,
    member_cooldown = 5
)]
pub async fn slap(
    ctx: Context<'_>,
    victim: serenity::User
) -> Result<(), Error> {
    let random_gif = {
        let mut rng = thread_rng();

        if &victim == ctx.author() {
            &ctx.data().self_slap_gifs[rng.gen_range(0..ctx.data().self_slap_gifs.len())]
        } else {
            &ctx.data().slap_gifs[rng.gen_range(0..ctx.data().slap_gifs.len())]
        }
    };

    let embed_msg = if &victim == ctx.author() {
        format!("{}, stop hitting yourself...stop hitting yourself", ctx.author())
    } else {
        format!("{} slaps {} around a bit with a large trout!", ctx.author(), victim)
    };

    let embed = serenity::CreateEmbed::new()
        .image(random_gif);

    ctx.send(poise::CreateReply::default().content(embed_msg).embed(embed)).await?;

    Ok(())
}

/// Cookiiiieeeesssssss
#[poise::command(
    slash_command,
    member_cooldown = 5
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
        format!("{}, NO! NO COOKIES FOR YOU!", ctx.author())
    } else {
        format!("{}, {} has given you a cookie!", victim, ctx.author())
    };

    let embed = serenity::CreateEmbed::new()
        .image(embed_image);

    ctx.send(poise::CreateReply::default().content(embed_msg).embed(embed)).await?;

    Ok(())
}