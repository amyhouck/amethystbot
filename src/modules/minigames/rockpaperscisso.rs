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

struct RPSPlayer {
    id: u64,
    choice: RPSChoices,
}

impl RPSPlayer {
    fn new(id: u64) -> RPSPlayer {
        Self {
            id,
            choice: RPSChoices::None
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
    let mut rps_game: [RPSPlayer; 2] = [
        RPSPlayer::new(ctx.author().id.get()),
        RPSPlayer::new(victim.id.get())
    ];

    // Handle message
    let embed_desc = format!("{}, you have been challenged to Rock, Paper, Scissors!\n\n Players, select a choice below! You have 1 minute!", victim);

    let rps_embed = serenity::CreateEmbed::new()
        .title("Rock, Paper, Scissors")
        .description(&embed_desc)
        .field(ctx.author().name.as_str(), rps_game[0].choice.to_string(), true)
        .field(victim.name.as_str(), rps_game[1].choice.to_string(), true)
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
        let player_num: usize = if press.user.id.get() == rps_game[0].id {
            0
        } else {
            1
        };

        if press.data.custom_id == rock_id {
            rps_game[player_num].choice = RPSChoices::Rock;

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else if press.data.custom_id == paper_id {
            rps_game[player_num].choice = RPSChoices::Paper;

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else if press.data.custom_id == scissors_id {
            rps_game[player_num].choice = RPSChoices::Scissors;

            press.create_response(ctx, serenity::CreateInteractionResponse::Acknowledge).await?;
        } else {
            continue;
        }

        // Embed handler
        if rps_game[0].choice != RPSChoices::None && rps_game[1].choice != RPSChoices::None {
            // Determine winner to update
            let winner = determine_winner_result(&rps_game[0].choice, &rps_game[1].choice);
            
            let embed_desc = match winner {
                1 => format!("<@{}> won! Congratulations!", rps_game[0].id),
                2 => format!("<@{}> won! Congratulations!", rps_game[1].id),
                _ => String::from("It was a tie!")
            };

            is_game_on = false;

            msg.edit(ctx, poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Rock, Paper, Scissors")
                    .description(embed_desc)
                    .field(ctx.author().name.as_str(), rps_game[0].choice.to_string(), true)
                    .field(victim.name.as_str(), rps_game[1].choice.to_string(), true)
                    .colour(0xFFFFFF)
                )
                .components(Vec::new())
            ).await?;

            break;
        }

        if rps_game[0].choice != RPSChoices::None || rps_game[1].choice != RPSChoices::None {
            let first_text = match rps_game[0].choice {
                RPSChoices::None => rps_game[0].choice.to_string(),
                _ => String::from("*Hidden*")
            };

            let second_text = match rps_game[1].choice {
                RPSChoices::None => rps_game[0].choice.to_string(),
                _ => String::from("*Hidden*")
            };

            msg.edit(ctx, poise::CreateReply::default()
                .embed(serenity::CreateEmbed::new()
                    .title("Rock, Paper, Scissors")
                    .description(&embed_desc)
                    .field(ctx.author().name.as_str(), first_text, true)
                    .field(victim.name.as_str(), second_text, true)
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