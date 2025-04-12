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
    guild_only,
    member_cooldown = 60
)]
pub async fn roulette(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // Grab game data
    let mut roulette = sqlx::query_as!(Roulette, "SELECT roulette_chamber, roulette_count FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();
    
    // If game is new, set it up
    if roulette.roulette_chamber == 0 {
        roulette.roulette_chamber = {
            let mut rng = thread_rng();
            rng.gen_range(1..=6)
        };
    }
    
    // Increment counter and run round
    roulette.roulette_count += 1;
    
    if roulette.roulette_count == roulette.roulette_chamber {
        let gif_url = "https://media3.giphy.com/media/v1.Y2lkPTc5MGI3NjExdGxqdWJwdTk5dGV6a2c4eTc3ZWhxdmc3MnZtcW5qMGRobHE5Yms1OCZlcD12MV9pbnRlcm5hbF9naWZfYnlfaWQmY3Q9Zw/3ohuPh9eyi4Av2DlM4/giphy.gif";
        let msg = String::from("You pick it up and...BANG!");
        
        let embed = serenity::CreateEmbed::new()
            .description(msg)
            .image(gif_url)
            .colour(0xFF0000);
            
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        
        let user_id = ctx.author().id.get();
        let roulette_win_query = format!("UPDATE guild_settings SET roulette_chamber = 0, roulette_count = 0 WHERE guild_id = {guild_id}
            UPDATE users SET roulette_deaths = roulette_deaths + 1 WHERE guild_id = {guild_id} AND user_id = {user_id}");
        
        sqlx::raw_sql(&roulette_win_query)
            .execute(&ctx.data().database)
            .await?;
    } else {
        let gif_url = "https://c.tenor.com/Zv53CH35UVAAAAAd/tenor.gif";
        let msg = format!("{}, you hear a click and nothing happens! You have survived the attempt.", ctx.author());
        
        let embed = serenity::CreateEmbed::new()
            .description(msg)
            .image(gif_url)
            .colour(0x00FF00);
            
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        
        sqlx::query!("UPDATE guild_settings SET roulette_chamber = ?, roulette_count = ? WHERE guild_id = ?", roulette.roulette_chamber, roulette.roulette_count, guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();
    }
    
    Ok(())
}