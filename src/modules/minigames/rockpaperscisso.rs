use crate::{Context, Error};
use poise::serenity_prelude as serenity;

// Choices
#[derive(PartialEq)]
enum RPSChoices {
    None,
    Rock,
    Paper,
    Scissors,
}

impl ToString for RPSChoices {
    fn to_string(&self) -> String {
        match self {
            RPSChoices::None => String::from("-------"),
            RPSChoices::Rock => String::from("Rock"),
            RPSChoices::Paper => String::from("Paper"),
            RPSChoices::Scissors => String::from("Scissors")
        }
    }
}

// Functions
fn determine_winner_result(
    challenger_choice: &RPSChoices,
    victim_choice: &RPSChoices
) -> u32 {
    if challenger_choice == victim_choice {
        return 0;
    }

    if *challenger_choice == RPSChoices::Rock {
        match victim_choice {
            RPSChoices::Paper => return 2,
            RPSChoices::Scissors => return 1,
            _ => return 3
        }
    } else if *challenger_choice == RPSChoices::Paper {
        match victim_choice {
            RPSChoices::Rock => return 1,
            RPSChoices::Scissors => return 2,
            _ => return 3
        }
    } else { // If challenger chose Scissors. This is to make the compiler happy and not yell at me about possibly missing an else
        match victim_choice {
            RPSChoices::Paper => return 1,
            RPSChoices::Rock => return 2,
            _ => return 3
        }
    }
}

/// Challenge someone to rock, paper, scissors!
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn rps(
    ctx: Context<'_>,
    #[description = "The challenged"] victim: serenity::User,
) -> Result<(), Error> {
    if ctx.author() == &victim {
        return Err("Uhhhhh no".into());
    }

    let guild_id = ctx.guild_id().unwrap().get();

    // Setup game data
    let mut challenger_choice = RPSChoices::None;
    let mut victim_choice = RPSChoices::None;

    // Handle message
    let embed_desc = format!("{}, you have been challenged to Rock, Paper, Scissors!\n\n Players, select a choice below! You have 1 minute!", victim);

    let rps_embed = serenity::CreateEmbed::new()
        .title("Rock, Paper, Scissors")
        .description(&embed_desc)
        .field(ctx.author().name.as_str(), challenger_choice.to_string(), true)
        .field(victim.name.as_str(), victim_choice.to_string(), true)
        .colour(0xFFFFFF);

    // Setup interaction data
    let ctx_id = ctx.id();
    let rock_id = format!("{ctx_id}rock");
    let paper_id = format!("{ctx_id}paper");
    let scissors_id = format!("{ctx_id}scissors");
    let players: [u64; 2] = [ctx.author().id.get(), victim.id.get()];
    let mut is_game_on = true;

    let buttons: Vec<serenity::CreateButton> = vec![
        serenity::CreateButton::new(&rock_id).label("Rock"),
        serenity::CreateButton::new(&paper_id).label("Paper"),
        serenity::CreateButton::new(&scissors_id).label("Scissors")
    ];
    let buttons = serenity::CreateActionRow::Buttons(buttons);

    // Send game and handle interactions
    let msg = ctx.send(poise::CreateReply::default()
        .content(format!("{}", victim))
        .embed(rps_embed)
        .components(vec![buttons])).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        // Filter players
        if !players.contains(&press.user.id.get()) {
            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
            continue;
        }

        // Handle buttons
        if press.data.custom_id == rock_id {
            if press.user.id.get() == ctx.author().id.get() {
                challenger_choice = RPSChoices::Rock;
            }

            if press.user.id.get() == victim.id.get() {
                victim_choice = RPSChoices::Rock;
            }

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else if press.data.custom_id == paper_id {
            if press.user.id.get() == ctx.author().id.get() {
                challenger_choice = RPSChoices::Paper;
            }

            if press.user.id.get() == victim.id.get() {
                victim_choice = RPSChoices::Paper;
            }

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else if press.data.custom_id == scissors_id {
            if press.user.id.get() == ctx.author().id.get() {
                challenger_choice = RPSChoices::Scissors;
            }

            if press.user.id.get() == victim.id.get() {
                victim_choice = RPSChoices::Scissors;
            }

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else {
            continue;
        }

        // Embed handler
        if challenger_choice != RPSChoices::None && victim_choice != RPSChoices::None {
            // Determine winner to update
            let winner = determine_winner_result(&challenger_choice, &victim_choice);
            
            let embed_desc = match winner {
                1 => format!("{} won! Congratulations!", ctx.author()),
                2 => format!("{} won! Congratulations!", victim),
                _ => String::from("It was a tie!")
            };

            is_game_on = false;

            msg.edit(ctx, poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Rock, Paper, Scissors")
                    .description(embed_desc)
                    .field(ctx.author().name.as_str(), challenger_choice.to_string(), true)
                    .field(victim.name.as_str(), victim_choice.to_string(), true)
                    .colour(0xFFFFFF)
                )
                .components(Vec::new())
            ).await?;

            break;
        }

        if challenger_choice != RPSChoices::None && victim_choice == RPSChoices::None {
            msg.edit(ctx, poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Rock, Paper, Scissors")
                    .description(&embed_desc)
                    .field(ctx.author().name.as_str(), "*Hidden*", true)
                    .field(victim.name.as_str(), victim_choice.to_string(), true)
                    .colour(0xFFFFFF)
                )
            ).await?;

            continue;
        }
        
        if challenger_choice == RPSChoices::None && victim_choice != RPSChoices::None {
            msg.edit(ctx, poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Rock, Paper, Scissors")
                    .description(&embed_desc)
                    .field(ctx.author().name.as_str(), challenger_choice.to_string(), true)
                    .field(victim.name.as_str(), "*Hidden*", true)
                    .colour(0xFFFFFF)
                )
            ).await?;

            continue;
        }
    }

    // Run if the game is still on
    if is_game_on {
        msg.edit(ctx, poise::CreateReply::default()
            .embed(serenity::CreateEmbed::new()
                .description("The game has timed out! :(")
                .colour(0xFFFFFF)
            )
            .components(Vec::new())
        ).await?;
    }

    Ok(())
}