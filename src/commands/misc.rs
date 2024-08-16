use crate::{log, Context, Error};
use crate::data::user_table_check;
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};
use chrono::Utc;

/// Slap slap slap, clap clap clap
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only,
)]
pub async fn slap(
    ctx: Context<'_>,
    victim: serenity::User
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

    let embed_msg = if &victim == ctx.author() {
        String::from("Stop hitting yourself...stop hitting yourself!")
    } else {
        format!("{} slaps you around a bit with a large trout!", ctx.author())
    };

    // Switch to using the victim's ID
    let victim_id = victim.id.get();

    // Build embed
    let embed = serenity::CreateEmbed::new()
        .description(embed_msg)
        .image(random_gif);

    ctx.send(poise::CreateReply::default().content(format!("<@{}>", victim_id)).embed(embed)).await?;

    // Stat handling
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    user_table_check(&ctx.data().database, guild_id, executioner_id).await; // - Check command executioner's existence
    user_table_check(&ctx.data().database, guild_id, victim_id).await;  // - Check victim's existence

    if ctx.author() != &victim {
        // Update executioner's stats
        sqlx::query!("UPDATE users SET slap_sent = slap_sent + 1 WHERE guild_id = ? AND user_id = ?", guild_id, executioner_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        // Update victim's stats
        sqlx::query!("UPDATE users SET slap_received = slap_received + 1 WHERE guild_id = ? AND user_id = ?", guild_id, victim_id)
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
    user_table_check(&ctx.data().database, guild_id, executioner_id).await; // - Check command executioner's existence
    user_table_check(&ctx.data().database, guild_id, victim_id).await;  // - Check victim's existence

    if ctx.author() != &victim {
        // Update executioner's stats
        sqlx::query!("UPDATE users SET cookie_sent = cookie_sent + 1 WHERE guild_id = ? AND user_id = ?", guild_id, executioner_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        // Update victim's stats
        sqlx::query!("UPDATE users SET cookie_received = cookie_received + 1 WHERE guild_id = ? AND user_id = ?", guild_id, victim_id)
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
    user_table_check(&ctx.data().database, guild_id, executioner_id).await; // - Check command executioner's existence
    user_table_check(&ctx.data().database, guild_id, victim_id).await;  // - Check victim's existence

    if ctx.author() != &victim {
        // Update executioner's stats
        sqlx::query!("UPDATE users SET tea_sent = tea_sent + 1 WHERE guild_id = ? AND user_id = ?", guild_id, executioner_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        // Update victim's stats
        sqlx::query!("UPDATE users SET tea_received = tea_received + 1 WHERE guild_id = ? AND user_id = ?", guild_id, victim_id)
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
        rng.gen_range(1..=15)
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
    user_table_check(&ctx.data().database, guild_id, executioner_id).await; // - Check command executioner's existence
    user_table_check(&ctx.data().database, guild_id, victim_id).await;  // - Check victim's existence

    // Update executioner's stats
    if ctx.author() != &victim {
        sqlx::query!("UPDATE users SET cake_sent = cake_sent + 1 WHERE guild_id = ? AND user_id = ?", guild_id, executioner_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        // Update victim's stats
        if glados == 9 {
            sqlx::query!("UPDATE users SET cake_glados = cake_glados + 1 WHERE guild_id = ? AND user_id = ?", guild_id, victim_id)
                .execute(&ctx.data().database)
                .await
                .unwrap();
        } else {
            sqlx::query!("UPDATE users SET cake_received = cake_received + 1 WHERE guild_id = ? AND user_id = ?", guild_id, victim_id)
                .execute(&ctx.data().database)
                .await
                .unwrap();
        }
    }

    Ok(())
}

/*---------------
| Bomb Minigame |
---------------*/

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ActiveBomb {
    pub guild_id: u64,
    pub target: u64,
    pub sender: u64,
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
        log::write_log(log::LogType::CommandError { ctx, error_msg: String::from("User tried to bomb themself.") });

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
        sender: ctx.author().id.get(),
        target: target.id.get(),
        wire_id,
        armed_time: Utc::now().format("%s").to_string().parse::<i32>().unwrap(),
        defuse_time: Utc::now().format("%s").to_string().parse::<i32>().unwrap() + 20,
        exploded: false,
        tries_remaining: 1,
    };

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

    let msg = ctx.send(poise::CreateReply::default()
        .content(format!("{target}"))
        .embed(bomb_embed)
        .components(vec![buttons])).await?;

    // Handle executioner stats and check for target
    user_table_check(&ctx.data().database, bomb.guild_id, bomb.sender).await;
    user_table_check(&ctx.data().database, bomb.guild_id, bomb.target).await;
    sqlx::query!("UPDATE users SET bomb_sent = bomb_sent + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.sender)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    // Button clicking event
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(20))
        .author_id(target.id)
        .await
    {
        // Update embed depending on cut attempt
        // - Dummy wire
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

        // - Success
        if press.data.custom_id == bomb.wire_id {
            // Handle interaction
            let mut msg = press.message.clone();
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("You have successfully defused the bomb!")
                        .image("https://media1.tenor.com/m/vu6SmJziKVUAAAAd/defusing-bomb-call-of-duty.gif")
                        .color(0x00FF00))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;

            // Handle stats for target
            sqlx::query!("UPDATE users SET bomb_defused = bomb_defused + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
                .execute(&ctx.data().database)
                .await
                .unwrap();

            break;
        }

        // - Wrong wire
        if press.data.custom_id != bomb.wire_id  {
            // Handle interaction
            let mut msg = press.message.clone();
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("Wrong wire!! ***KABOOM***")
                        .image("https://media1.tenor.com/m/_TQcegphP7MAAAAd/boom-bomb.gif")
                        .color(0xFF0000))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;

            // Handle target stats
            sqlx::query!("UPDATE users SET bomb_failed = bomb_failed + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
                .execute(&ctx.data().database)
                .await
                .unwrap();

            break;
        }
    }

    // Check if bomb is still active after timeout
    if !bomb.exploded {
        msg.edit(ctx, poise::CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .description("***KABOOM*** You ran out of time!")
                .image("https://media1.tenor.com/m/4H_YjM4P2IkAAAAC/bh187-spongebob.gif")
                .color(0xFF0000)
            )
            .components(Vec::new())
        ).await?;

        sqlx::query!("UPDATE users SET bomb_failed = bomb_failed + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

    Ok(())
}