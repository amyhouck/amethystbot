use crate::{Context, Error};
use poise::serenity_prelude as serenity;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

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
    Tea
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
            GIFType::Tea => write!(f, "tea")
        }
    }
}

#[allow(dead_code)]
pub struct CustomGif {
    pub guild_id: u64,
    pub gif_type: String,
    pub gif_id: u32,
    pub description: Option<String>,
    pub filename: String
}

pub enum GIFDBQueryType {
    Normal,
    SingleRandom
}

//--------------------
// Functions
//--------------------
// Convert GIF vector into String vector
fn create_gif_pages(gifs: Vec<CustomGif>) -> Vec<String> {
    let mut page_content = String::new();
    let mut pages: Vec<String> = Vec::new();

    for (i, gif) in gifs.iter().enumerate() {
        let filename = URL_SAFE_NO_PAD.decode(&gif.filename).unwrap();
        let filename = String::from_utf8(filename).unwrap();
        
        page_content = match &gif.description {
            Some(desc) => {
                format!("{page_content}**{}.** *{filename}* -- {desc}\n\n",
                    i + 1,
                )
            },
            None => {
                format!("{page_content}**{}.** *{filename}*\n\n",
                    i + 1,
                )
            }
        };

        // Split content into more pages as necessary
        if i + 1 == gifs.len() {
            pages.push(page_content);
            break;
        }

        if (i + 1) % 5 == 0 {
            pages.push(page_content);
            page_content = String::new();
        }
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

//--------------------
// Commands
//--------------------
/// Customize birthdays and miscellaneous commands with your own GIFs.
#[poise::command(
    slash_command,
    guild_only,
    check = "check_gif_role",
    member_cooldown = 20,
)]
pub async fn addgif(
    ctx: Context<'_>,
    #[description = "The command to add a GIF for"]
    command: GIFType,
    
    #[description = "The name of you want to give the GIF"]
    #[max_length = 30]
    name: String,

    #[description = "The GIF you want to upload"]
    gif: serenity::Attachment,
    
    #[description = "An optional description to help identify the GIF in a list"]
    #[max_length = 100]
    description: Option<String>
) -> Result<(), Error> {
    ctx.defer().await?;
    
    let guild_id = ctx.guild_id().unwrap().get();
    let gif_type = command.to_string();
    
    // Validate if the attachment is a GIF
    if gif.content_type != Some(String::from("image/gif")) {
        return Err("You can only upload GIFs!".into());
    }
    
    // Make sure they don't already have 10 GIFs
    let gif_id = sqlx::query!("SELECT MAX(gif_id) AS gif_id FROM custom_gifs WHERE guild_id = ? AND gif_type = ?", guild_id, gif_type)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .gif_id
        .unwrap_or(0);
        
    if gif_id >= 10 {
        return Err("The server has reached the limit for the amount of GIFs saved for this command!".into());
    }
    
    // Check file already exists with that name after encoding into URL-Safe Base64
    // I am using URL-Safe Base64 encoding so people can have some freedom as to what they name
    // their GIFs when they're saved. It also saves me the headache of validating the names
    let enc_name = URL_SAFE_NO_PAD.encode(&name);
    let dir = format!("CustomGIFs/{guild_id}/{gif_type}/");
    fs::create_dir_all(&dir).await?;
    let path = format!("{dir}{enc_name}.gif");
    match fs::try_exists(&path).await {
        Ok(exists) => {
            if exists {
                return Err("A GIF with that name is already saved!".into());
            }
        },
        Err(_) => return Err("Unable to determine if that GIF exists!".into())
    }
    
    // Save GIF to files
    let content = match gif.download().await {
        Ok(data) => data,
        Err(_) => return Err("There was an error trying to save the GIF!".into())
    };
    
    let mut file = fs::File::create(&path).await?;
    file.write_all(&content).await?;
    
    // Save information to database
    sqlx::query!("INSERT INTO custom_gifs (guild_id, gif_type, gif_id, description, filename) VALUES (?, ?, ?, ?, ?)",
        guild_id, gif_type, gif_id + 1, description, &enc_name)
        .execute(&ctx.data().database)
        .await
        .unwrap();
    
    // Create embed to show it saved
    let filename = format!("{enc_name}.gif");
    let saved_gif = serenity::CreateAttachment::bytes(content, filename);
    
    let embed = serenity::CreateEmbed::new()
        .description(format!("Successfully saved the GIF: {name} for {command}"))
        .colour(0x0b4a6f)
        .image(format!("attachment://{enc_name}.gif"));
        
    let msg = poise::CreateReply::default()
        .embed(embed)
        .attachment(saved_gif);
        
    ctx.send(msg).await?;
    

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

/// List the custom GIFs set for a command
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn listgifs(
    ctx: Context<'_>,

    #[description = "The command with custom GIFs"]
    gif_type: GIFType
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let gif_type_string = gif_type.to_string();

    // Grab relevant GIFs, return error if empty vector
    let gifs = grab_custom_gifs(&ctx.data().database, &gif_type, guild_id, GIFDBQueryType::Normal).await;

    if gifs.is_empty() {
        return Err(format!("No GIFs were found under \"{gif_type_string}\"").into());
    }

    // Create GIF embed
    let gif_pages = create_gif_pages(gifs);
    let mut page_num = 0;

    let embed = serenity::CreateEmbed::new()
        .description(&gif_pages[page_num])
        .colour(0x0b4a6f)
        .title(format!("Custom GIFs for \"{gif_type_string}\""));

    let mut reply_obj = poise::CreateReply::default()
        .embed(embed);

    if gif_pages.len() > 1 {
        let ctx_id = ctx.id();
        let prev_id = format!("{ctx_id}prev");
        let next_id = format!("{ctx_id}next");

        let buttons: Vec<serenity::CreateButton> = vec![
            serenity::CreateButton::new(&prev_id).label("Previous"),
            serenity::CreateButton::new(&next_id).label("Next")
        ];
        let buttons = serenity::CreateActionRow::Buttons(buttons);

        reply_obj = reply_obj.components(vec![buttons]);
        ctx.send(reply_obj).await?;

        // Handle button interactions
        while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
            .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
            .timeout(std::time::Duration::from_secs(1800))
            .await {
                if press.data.custom_id == prev_id {
                    page_num = page_num.checked_sub(1).unwrap_or(gif_pages.len() - 1)
                } else if press.data.custom_id == next_id {
                    page_num += 1;
                    if page_num >= gif_pages.len() { page_num = 0; }
                } else {
                    continue;
                }

                press.create_response(
                    ctx.serenity_context(),
                    serenity::CreateInteractionResponse::UpdateMessage(
                        serenity::CreateInteractionResponseMessage::new()
                            .embed(serenity::CreateEmbed::new()
                                .description(&gif_pages[page_num])
                                .colour(0x0b4a6f)
                                .title(format!("Custom GIFs for \"{gif_type_string}\""))
                            )
                    )
                ).await?;
        }
    } else {
        ctx.send(reply_obj).await?;
    }

    Ok(())
}

/// View a saved GIF
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 10
)]
pub async fn viewgif(
    ctx: Context<'_>,
    
    #[description = "The GIF type"]
    gif_type: GIFType,
    
    #[description = "The GIF ID"]
    id: u32
) -> Result<(), Error> {
    ctx.defer().await?;
    let guild_id = ctx.guild_id().unwrap().get();
    
    // Run GIF query and validate existence
    let gif = sqlx::query_as!(CustomGif,"SELECT * FROM custom_gifs WHERE guild_id = ? AND gif_id = ? AND gif_type = ?",
        guild_id, id, gif_type.to_string())
        .fetch_optional(&ctx.data().database)
        .await
        .unwrap();
        
    let gif = match gif {
        Some(g) => g,
        None => {
            let msg = format!("No GIF exists with that ID for \"{gif_type}\"!");
            return Err(msg.into());
        }
    };
    
    // Grab GIF file and send embed
    let path = format!("CustomGIFs/{guild_id}/{gif_type}/{}.gif", &gif.filename);
    let attachment = serenity::CreateAttachment::path(path).await?;
    let dec_name = String::from_utf8(URL_SAFE_NO_PAD.decode(&gif.filename)?).unwrap();
    
    let title = format!("CustomGIF - {dec_name}");
    let description = format!("
        **Type:** {}
        **Description:** {}",
        gif.gif_type,
        gif.description.unwrap_or(String::from("*None*"))
    );
    
    let embed = serenity::CreateEmbed::new()
        .title(title)
        .description(description)
        .colour(0x0b4a6f)
        .image(format!("attachment://{}.gif", &gif.filename));
        
    let msg = poise::CreateReply::default()
        .embed(embed)
        .attachment(attachment);
        
    ctx.send(msg).await?;
    
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