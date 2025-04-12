use crate::{customgifs::{grab_custom_gifs, GIFDBQueryType, GIFType}, user_table_check, Context, Error};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};
use chrono::Utc;

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
    #[description = "The user you'd like to bomb."] target: serenity::User
) -> Result<(), Error> {
    // Disable stat queries if targeting self
    let targeting_self = ctx.author() == &target;

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

    // Handle stats and check for target
    if !targeting_self {
        user_table_check(ctx, &target).await;
        sqlx::query!("UPDATE users SET bomb_sent = bomb_sent + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.sender)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }

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
            let gif = grab_custom_gifs(&ctx.data().database, &GIFType::BombDefuse, bomb.guild_id, GIFDBQueryType::SingleRandom).await;
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("You have successfully defused the bomb!")
                        .image(&gif[0].gif_url)
                        .color(0x00FF00))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;

            // Handle stats for target
            if !targeting_self {
                sqlx::query!("UPDATE users SET bomb_defused = bomb_defused + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
                    .execute(&ctx.data().database)
                    .await
                    .unwrap();
            }

            break;
        }

        // - Wrong wire
        if press.data.custom_id != bomb.wire_id  {
            // Handle interaction
            let mut msg = press.message.clone();
            let gif = grab_custom_gifs(&ctx.data().database, &GIFType::BombFailure, bomb.guild_id, GIFDBQueryType::SingleRandom).await;
            msg.edit(ctx, 
                serenity::EditMessage::new()
                    .embed(serenity::CreateEmbed::new()
                        .description("Wrong wire!! ***KABOOM***")
                        .image(&gif[0].gif_url)
                        .color(0xFF0000))
                    .components(Vec::new())
            ).await?;
            bomb.exploded = true;
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;

            // Handle target stats
            if !targeting_self {
                sqlx::query!("UPDATE users SET bomb_failed = bomb_failed + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
                    .execute(&ctx.data().database)
                    .await
                    .unwrap();
            }

            break;
        }
    }

    // Check if bomb is still active after timeout
    if !bomb.exploded {
        let gif = grab_custom_gifs(&ctx.data().database, &GIFType::BombTime, bomb.guild_id, GIFDBQueryType::SingleRandom).await;
        msg.edit(ctx, poise::CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .description("***KABOOM*** You ran out of time!")
                .image(&gif[0].gif_url)
                .color(0xFF0000)
            )
            .components(Vec::new())
        ).await?;

        if !targeting_self {
            sqlx::query!("UPDATE users SET bomb_failed = bomb_failed + 1 WHERE guild_id = ? AND user_id = ?", bomb.guild_id, bomb.target)
                .execute(&ctx.data().database)
                .await
                .unwrap();
        }
    }

    Ok(())
}