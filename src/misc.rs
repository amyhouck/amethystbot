use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};
use chrono::Utc;

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

/*---------------
| Bomb Minigame |
---------------*/

#[derive(Clone, Debug)]
pub struct ActiveBomb {
    pub guild_id: u64,
    pub target: u64,
    pub wire_id: String,
    pub armed_time: i32,
    pub defuse_time: i32,
    pub exploded: bool,
    pub tries_remaining: i32,
}

/// Start the bomb minigame
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only
)]
pub async fn bomb(
    ctx: Context<'_>,
    target: serenity::User
) -> Result<(), Error> {
    // Don't allow attacking self
    if &target == ctx.author() {
        return Err("You can't do that!".into());
    }

    // Build bomb
    let ctx_id = ctx.id();
    let wire_id = {
        let mut rng = thread_rng();

        match rng.gen_range(0..=4) {
            0 => format!("{ctx_id}red"),
            1 => format!("{ctx_id}green"),
            2 => format!("{ctx_id}white"),
            3 => format!("{ctx_id}black"),
            4 => format!("{ctx_id}blue"),
            _ => format!("{ctx_id}red"),
        }
    };

    let mut bomb = ActiveBomb {
        guild_id: ctx.guild_id().unwrap().get(),
        target: target.id.get(),
        wire_id,
        armed_time: Utc::now().format("%s").to_string().parse::<i32>().unwrap(),
        defuse_time: Utc::now().format("%s").to_string().parse::<i32>().unwrap() + 20,
        exploded: false,
        tries_remaining: 1,
    };
    println!("{:#?}", bomb);

    // Create and send message
    let button_ids: [String; 5] = [
        format!("{ctx_id}red"),
        format!("{ctx_id}green"),
        format!("{ctx_id}white"),
        format!("{ctx_id}black"),
        format!("{ctx_id}blue"),
    ];

    let buttons: Vec<serenity::CreateButton> = vec![
        serenity::CreateButton::new(&button_ids[0]).label("Red"),
        serenity::CreateButton::new(&button_ids[1]).label("Green"),
        serenity::CreateButton::new(&button_ids[2]).label("White"),
        serenity::CreateButton::new(&button_ids[3]).label("Black"),
        serenity::CreateButton::new(&button_ids[4]).label("Blue"),
    ];

    let buttons = serenity::CreateActionRow::Buttons(buttons);

    let bomb_embed = serenity::CreateEmbed::new()
        .description(format!("{} has sent you a bomb! Defuse it quickly! You have 20 seconds!", ctx.author().name))
        .color(0xFF0000);

    ctx.send(poise::CreateReply::default()
        .content(format!("{target}"))
        .embed(bomb_embed)
        .components(vec![buttons])).await?;

    // Button clicking event
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(20))
        .author_id(target.id)
        .await
    {
        // Update embed depending on cut attempt
        if bomb.tries_remaining > 0 && press.data.custom_id != bomb.wire_id {
            let mut msg = press.message.clone();
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("That was a dummy wire! You have 1 more chance!")
                        .color(0x00FF00))
            ).await?;
            bomb.tries_remaining -= 1;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
            continue;
        }

        if press.data.custom_id == bomb.wire_id {
            let mut msg = press.message.clone();
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("You have successfully defused the bomb!")
                        .color(0x00FF00))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
            break;
        }

        if press.data.custom_id != bomb.wire_id  {
            let mut msg = press.message.clone();
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("Wrong wire!! ***KABOOM***")
                        .color(0xFF0000))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
            break;
        }
    }

    if !bomb.exploded {
        ctx.say("You ran out of time :(").await?;
    }

    Ok(())
}