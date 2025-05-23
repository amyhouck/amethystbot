use crate::{Context, Error};
use poise::serenity_prelude as serenity;

//--------------------
// Data
//--------------------
#[derive(poise::ChoiceParameter)]
pub enum GIFType {
    Birthday,
    Cake,
    Cookie,
    CookieSelf,
    Slap,
    SlapSelf,
    Tea,
    Hug,
    BombTime,
    BombFailure,
    BombDefuse,
}

impl std::fmt::Display for GIFType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GIFType::Birthday => write!(f, "birthday"),
            GIFType::Cake => write!(f, "cake"),
            GIFType::Cookie => write!(f, "cookie"),
            GIFType::CookieSelf => write!(f, "cookie_self"),
            GIFType::Slap => write!(f, "slap"),
            GIFType::SlapSelf => write!(f, "slap_self"),
            GIFType::Tea => write!(f, "tea"),
            GIFType::Hug => write!(f, "hug"),
            GIFType::BombTime => write!(f, "bomb_time"),
            GIFType::BombFailure => write!(f, "bomb_failure"),
            GIFType::BombDefuse => write!(f, "bomb_defuse")
        }
    }
}

#[allow(dead_code)]
pub struct CustomGif {
    pub guild_id: u64,
    pub gif_type: String,
    pub gif_id: u32,
    pub gif_url: String,
    pub gif_name: String,
}

pub enum GIFDBQueryType {
    Normal,
    SingleRandom
}

//--------------------
// Functions
//--------------------
// Convert GIF vector into String vector
fn create_gif_pages(gifs: Vec<CustomGif>) -> Vec<serenity::CreateEmbed> {
    let mut pages: Vec<serenity::CreateEmbed> = Vec::new();
    
    for (i, custom_gif) in gifs.iter().enumerate() {
        let description = format!("**Name:** {}\n**URL:** {}", custom_gif.gif_name, &custom_gif.gif_url);
        
        let embed = serenity::CreateEmbed::new()
            .title(format!("{} GIFs {}/{}", custom_gif.gif_type, i + 1, gifs.len()))
            .description(description)
            .image(&custom_gif.gif_url)
            .colour(0x0b4a6f);
            
        pages.push(embed);
    }
    
    pages
}

// Grab specific type of GIFs for the server
pub async fn grab_custom_gifs(
    database: &sqlx::MySqlPool,
    gif_type: &GIFType,
    guild_id: u64,
    query_type: GIFDBQueryType
) -> Vec<CustomGif> {
    // Keep using query_as!() and .fetch_all() to keep the Vector type and not have to deal with whether it's a single item or not in this function.
    match query_type {
        GIFDBQueryType::Normal => { 
            sqlx::query_as!(CustomGif, "SELECT * FROM custom_gifs WHERE guild_id = ? AND gif_type = ?", guild_id, gif_type.to_string())
                .fetch_all(database)
                .await
                .unwrap()
        },
        GIFDBQueryType::SingleRandom => {
            sqlx::query_as!(CustomGif, "SELECT * FROM custom_gifs WHERE guild_id = ? AND gif_type = ? ORDER BY RAND() LIMIT 1", guild_id, gif_type.to_string())
                .fetch_all(database)
                .await
                .unwrap()
        }
    }
}

// Check if GIF commands require roles
async fn check_gif_role(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let role_id = sqlx::query!("SELECT custom_gifs_required_role FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .custom_gifs_required_role.unwrap_or(0);

    if role_id == 0 {
        return Ok(true);
    }

    let role_id = serenity::RoleId::new(role_id);
    let guild_roles = ctx.guild_id().unwrap().roles(&ctx.http()).await.unwrap();
    let role = match guild_roles.get(&role_id) {
        Some(r) => r,
        None => return Err("A role is set to be required for this command, but it doesn't exist!".into())
    };

    if !ctx.author().has_role(ctx.http(), guild_id, role_id).await.unwrap() {
        ctx.send(
    poise::CreateReply::default()
                .content(format!("You must have the '{}' role to run this command!", role.name))
                .ephemeral(true)
        ).await?;

        return Ok(false);
    }

    Ok(true)
}

// Grab random GIF attachment.
pub async fn grab_misc_gif(
    database: &sqlx::MySqlPool,
    guild_id: u64,
    gif_type: &GIFType
) -> Option<String> {
    let random_gif = grab_custom_gifs(database, gif_type, guild_id, GIFDBQueryType::SingleRandom).await;
    
    if !random_gif.is_empty() {
        let url = random_gif[0].gif_url.to_owned();
        Some(url)
    } else {
        None
    }
}

//--------------------
// Commands
//--------------------
/// Customize birthdays and miscellaneous commands with your own GIFs.
#[poise::command(
    slash_command,
    guild_only,
    check = "check_gif_role",
    member_cooldown = 5,
)]
pub async fn addgif(
    ctx: Context<'_>,
    #[description = "The command to add a GIF for"]
    command: GIFType,

    #[description = "An identifying name for the GIF"]
    #[max_length = 30]
    gif_name: String,
    
    #[description = "The URL for the GIF"]
    gif_url: String
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let gif_type = command.to_string();

    // Set future GIF ID
    let gif_id = sqlx::query!("SELECT MAX(gif_id) AS gif_id FROM custom_gifs WHERE guild_id = ? AND gif_type = ?", guild_id, gif_type)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .gif_id
        .unwrap_or(0);
    
    // Insert into DB
    sqlx::query!("INSERT INTO custom_gifs (guild_id, gif_type, gif_id, gif_url, gif_name) VALUES (?, ?, ?, ?, ?)", guild_id, gif_type, gif_id + 1, gif_url, gif_name)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Registered a new GIF for \"{gif_type}\"! {gif_url}")).await?;

    Ok(())
}

/// Remove a custom GIF from a command
#[poise::command(
    slash_command,
    guild_only,
    check = "check_gif_role",
    member_cooldown = 5,
)]
pub async fn delgif(
    ctx: Context<'_>,
    #[description = "The command that has the GIF you want to remove"]
    command: GIFType,

    #[description = "The GIF ID"]
    gif_id: u32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let gif_type = command.to_string();

    // Determine if GIF ID exists
    let count = sqlx::query!("SELECT COUNT(gif_id) AS count FROM custom_gifs WHERE guild_id = ? AND gif_type = ? AND gif_id = ?", guild_id, gif_type, gif_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .count;

    if count < 1 {
        return Err(format!("No GIF has been registered with that ID under \"{gif_type}\"!").into());
    }

    // Remove GIF from database and then reset IDs
    sqlx::query!("DELETE FROM custom_gifs WHERE guild_id = ? AND gif_type = ? AND gif_id = ?", guild_id, gif_type, gif_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    sqlx::query!("UPDATE custom_gifs SET gif_id = gif_id - 1 WHERE guild_id = ? AND gif_type = ? AND gif_id > ?", guild_id, gif_type, gif_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Deleted GIF from \"{gif_type}\"!")).await?;

    Ok(())
}

/// List the custom GIFs set for this server
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn listgifs(
    ctx: Context<'_>,
    
    #[description = "The type of GIF to list"]
    gif_type: GIFType
) -> Result<(), Error> {
    // Grab relevant GIFs, return error if empty
    let gifs = grab_custom_gifs(&ctx.data().database, &gif_type, ctx.guild_id().unwrap().get(), GIFDBQueryType::Normal).await;
    
    if gifs.is_empty() {
        return Err(format!("No GIFs were found for \"{gif_type}\"").into());
    }
    
    // Run interaction for the gif pages
    let gif_pages = create_gif_pages(gifs);
    let mut reply = poise::CreateReply::default()
        .embed(gif_pages[0].clone());
        
    if gif_pages.len() > 1 {
        let ctx_id = ctx.id();
        let next_id = format!("{ctx_id}next");
        let prev_id = format!("{ctx_id}prev");
        let mut page_num: usize = 0;
        
        let buttons: Vec<serenity::CreateButton> = vec![
            serenity::CreateButton::new(&prev_id).label("Previous"),
            serenity::CreateButton::new(&next_id).label("Next")  
        ];
        
        let buttons = serenity::CreateActionRow::Buttons(buttons);
        reply = reply.components(vec![buttons]);
        
        ctx.send(reply).await?;
        
        while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
            .timeout(std::time::Duration::from_secs(600))
            .await {
                if press.data.custom_id == prev_id {
                    page_num = page_num.checked_sub(1).unwrap_or(gif_pages.len() - 1);
                } else if press.data.custom_id == next_id {
                    page_num = if page_num == gif_pages.len() - 1 {
                        0  
                    } else {
                        page_num + 1
                    };
                } else {
                    continue;
                }
                
                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(gif_pages[page_num].clone())
                    )
                ).await?;
        }
    } else {
        ctx.send(reply).await?;
    }
    
    Ok(())
}

/// Set a required role to run custom gif commands
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    member_cooldown = 5,
)]
pub async fn setgifrole(
    ctx: Context<'_>,
    #[description = "The role to require"] gif_role: Option<serenity::Role>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Remove the role if none is specified
    if gif_role.is_none() {
        sqlx::query!("UPDATE guild_settings SET custom_gifs_required_role = NULL WHERE guild_id = ?", guild_id)
            .execute(&ctx.data().database)
            .await
            .unwrap();

        ctx.say("No longer requiring a role to modify custom GIFs!").await?;
        return Ok(());
    }

    // Add role otherwise
    let role_id = gif_role.as_ref().unwrap().id.get();

    sqlx::query!("UPDATE guild_settings SET custom_gifs_required_role = ? WHERE guild_id = ?", role_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Now requiring the {} role to use custom GIF commands.", gif_role.unwrap().name)).await?;

    Ok(())
}