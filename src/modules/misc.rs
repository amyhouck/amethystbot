use crate::{Context, Error};
use crate::data::user_table_check;
use crate::customgifs::{grab_misc_gif, GIFType};
use poise::serenity_prelude as serenity;
use rand::{Rng, thread_rng};

#[derive(Clone, Copy)]
enum MiscCommand {
    Slap,
    Tea,
    Cake,
    Cookie
}

impl std::fmt::Display for MiscCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiscCommand::Slap => write!(f, "slap"),
            MiscCommand::Tea => write!(f, "tea"),
            MiscCommand::Cake => write!(f, "cake"),
            MiscCommand::Cookie => write!(f, "cookie")
        }
    }
}

impl MiscCommand {
    fn determine_gif_type(self, target_self: bool) -> GIFType {
        match self {
            MiscCommand::Slap => {
                if target_self {
                    GIFType::SlapSelf
                } else {
                    GIFType::Slap
                }
            },
            MiscCommand::Cookie => {
                if target_self {
                    GIFType::CookieSelf
                } else {
                    GIFType::Cookie
                }
            },
            MiscCommand::Tea => GIFType::Tea,
            MiscCommand::Cake => GIFType::Cake
        }
    }
}

// Determine if user is accepting pings
async fn is_user_pingable(
    database: &sqlx::MySqlPool,
    guild_id: u64,
    user_id: u64
) -> bool {
    let ping = sqlx::query!("SELECT command_ping FROM user_settings WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(database)
        .await
        .unwrap()
        .command_ping;
        
    ping == 1
}

async fn handle_user_stats(
    command: &MiscCommand,
    ctx: Context<'_>,
    victim: serenity::User,
    glados_trigger: bool
) {
    let guild_id = ctx.guild_id().unwrap().get();
    let executioner_id = ctx.author().id.get();
    let victim_id = victim.id.get();
        
    let query = if glados_trigger {
        format!("UPDATE users SET cake_sent = cake_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET cake_glados = cake_glados + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}")
    } else {
        format!("UPDATE users SET {command}_sent = {command}_sent + 1 WHERE guild_id = {guild_id} AND user_id = {executioner_id};
            UPDATE users SET {command}_received = {command}_received + 1 WHERE guild_id = {guild_id} AND user_id = {victim_id}")
    };
        
    sqlx::raw_sql(&query)
        .execute(&ctx.data().database)
        .await
        .unwrap();
}

/// Slap slap slap, clap clap clap
#[poise::command(
    slash_command,
    member_cooldown = 5,
    guild_only,
)]
pub async fn slap(
    ctx: Context<'_>,
    #[description = "The user you'd like to slap."] victim: serenity::User
) -> Result<(), Error> {
    misc_container(ctx, MiscCommand::Slap, victim).await?;

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
    misc_container(ctx, MiscCommand::Cookie, victim).await?;

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
    misc_container(ctx, MiscCommand::Tea, victim).await?;

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
    misc_container(ctx, MiscCommand::Cake, victim).await?;

    Ok(())
}

async fn misc_container(
    ctx: Context<'_>,
    command: MiscCommand,
    victim: serenity::User,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    
    // Determine GIF
    let gif_type = command.determine_gif_type(ctx.author() == &victim);
    
    let mut random_gif = grab_misc_gif(&ctx.data().database, guild_id, &gif_type).await;
    
    // Determine embed message
    let mut glados_trigger= false;
    let mut msg = match gif_type {
        GIFType::Tea => {
            if ctx.author() == &victim {
                String::from("You have received some tea!")
            } else {
                format!("{} has given you some tea!", ctx.author())
            }
        },
        GIFType::Slap => format!("{} slaps you around a bit with a large trout!", ctx.author()),
        GIFType::SlapSelf => String::from("Stop hitting yourself! Stop hitting yourself!"),
        GIFType::Cake => {
            let mut rng = thread_rng();
            let glados = rng.gen_range(1..=13);
            
            if glados == 9 {
                random_gif = Some(String::from("https://media1.tenor.com/m/I1ZYLNNNEGQAAAAC/portal-glados.gif"));
                glados_trigger = true;
                String::from("***The cake is a lie***")
            } else {
                format!("{} has given you some cake! Hope you like it!", ctx.author())
            }
        },
        GIFType::Cookie => format!("You have received a cookie from {}!", ctx.author()),
        GIFType::CookieSelf => String::from("NO! No cookies for you!"),
        _ => String::new()
    };
    
    // Build and send embed
    let mut embed = serenity::CreateEmbed::new();
    let mut reply = poise::CreateReply::default();
    
    // - Ping user if allowed
    if is_user_pingable(&ctx.data().database, guild_id, victim.id.get()).await {
        reply = reply.content(format!("{victim}"));
    } else {
        msg = format!("{victim}: {msg}");
    }
        
    // - Add GIF if it exists
    if let Some(url) = random_gif {
        embed = embed.image(url);
    }
    embed = embed.description(msg);
    reply = reply.embed(embed);
    
    ctx.send(reply).await?;
    
    // Handle stats
    if ctx.author() != &victim {
        user_table_check(ctx, &victim).await;
        handle_user_stats(&command, ctx, victim, glados_trigger).await;
    }
    
    Ok(())
}