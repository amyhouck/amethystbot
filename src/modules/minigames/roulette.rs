use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

#[derive(Debug)]
struct Roulette {
    roulette_chamber: u8,
    roulette_count: u8
}

/// Try your luck with Russian Roulette
#[poise::command(
    slash_command,
    guild_only
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // Grab game data
    let mut roulette = sqlx::query_as!(Roulette, "SELECT roulette_chamber, roulette_count FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();
    
    // If game is new, set it up
    println!("OLD: {:?}", roulette);
    if roulette.roulette_chamber == 0 {
        roulette.roulette_chamber = {
            let mut rng = thread_rng();
            rng.gen_range(1..=6)
        };
    }
    
    // Increment counter and run round
    roulette.roulette_count += 1;
    println!("NEW: {:?}", roulette);
    
    if roulette.roulette_count == roulette.roulette_chamber {
        ctx.say("Bang!").await?;
        
        sqlx::query!("UPDATE guild_settings SET roulette_chamber = 0, roulette_count = 0 WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    } else {
        ctx.say("Click!").await?;
        
        sqlx::query!("UPDATE guild_settings SET roulette_chamber = ?, roulette_count = ? WHERE guild_id = ?", roulette.roulette_chamber, roulette.roulette_count, guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }
    
    Ok(())
}