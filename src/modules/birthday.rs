use crate::{Data, Context, Error};
use crate::customgifs::{grab_custom_gifs, GIFType, GIFDBQueryType};
use poise::serenity_prelude as serenity;
use chrono::Utc;
use rand::{thread_rng, Rng};

//--------------------
// Data
//--------------------
#[derive(sqlx::FromRow, Debug)]
pub struct Birthday {
    guild_id: u64,
    user_id: u64,
    birthday: u8,
    birthmonth: u8,
    nickname: Option<String>
}

const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

//--------------------
// Function library
//--------------------
// Check if birthday channel is set before running a command
async fn bday_channel_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    let settings = sqlx::query!("SELECT birthday_channel FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    match settings.birthday_channel {
        Some(_) => Ok(true),
        None => {
            ctx.send(poise::CreateReply::default()
                .content("You must select a channel to post birthday announcements in before using this command!")
                .ephemeral(true)
            ).await?;
            Ok(false)
        }
    }
}

// Determine which username to use
async fn determine_username (
    ctx: &serenity::Context,
    database: &sqlx::MySqlPool,
    bday_nick: Option<String>,
    guild_id: u64,
    user_id: u64
) -> String {
    if bday_nick.is_none() {
        // Try to grab from user row
        let user_display_name = sqlx::query!("SELECT display_name FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .fetch_optional(database)
            .await
            .unwrap();

        match user_display_name {
            Some(record) => return record.display_name,
            None => return serenity::UserId::new(user_id).to_user(&ctx).await.unwrap().name
        }
    }

    bday_nick.unwrap()
}

//--------------------
// Commands
//--------------------
#[poise::command(
    slash_command,
    guild_only,
    subcommands("add", "remove", "edit", "setchannel", "info", "setrole", "list")
)]
pub async fn bday(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Add a user's birthday
#[poise::command(
    slash_command,
    check = "bday_channel_check",
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "User to add."] user: serenity::User,
    #[description = "Birthmonth."]
    #[min = 1_u8]
    #[max = 12_u8] month: u8,
    #[description = "Birthday."]
    #[min = 1_u8]
    #[max = 31_u8] day: u8,
    #[description = "A nickname for the user."]
    #[max_length = 30] nickname: Option<String>,
) -> Result<(), Error> {
    let birthday = Birthday {
        guild_id: ctx.guild_id().unwrap().get(),
        user_id: user.id.get(),
        birthday: day,
        birthmonth: month,
        nickname,
    };

    let count = sqlx::query!("SELECT COUNT(user_id) AS count FROM birthday WHERE guild_id = ? AND user_id = ?", birthday.guild_id, birthday.user_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count != 0 {
        return Err(format!("{}'s birthday is already saved!", user.name).into());
    }

    sqlx::query!("INSERT INTO birthday (guild_id, user_id, birthday, birthmonth, nickname) VALUES (?, ?, ?, ?, ?)", birthday.guild_id, birthday.user_id, birthday.birthday, birthday.birthmonth, birthday.nickname)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Successfully added {}'s birthday!", user.name)).await?;

    Ok(())
}

/// Remove a user's birthday
#[poise::command(
    slash_command,
    check = "bday_channel_check",
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "User to remove."] user: Option<serenity::User>,
    #[description = "Manually enter user ID if user is no longer in the guild."] 
    #[max_length = 20] user_id: Option<String>
) -> Result<(), Error> {
    if user.is_none() && user_id.is_none() {
        return Err("You must choose a user or manually enter the user ID!".into());
    }
    let guild_id = ctx.guild_id().unwrap().get();

    let user_id = match &user {
        Some(u) => u.id.get(),
        None => {
            match user_id.unwrap().parse::<u64>() {
                Ok(i) => i,
                Err(_) => return Err("You must enter a valid UserID!".into())
            }
        }
    };

    let count = sqlx::query!("SELECT COUNT(user_id) AS count FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count == 0 {
        let msg = if user.is_some() {
            format!("No birthday found for {}!", user.unwrap().name)
        } else {
            format!("No birthday found matching ID: {user_id}")
        };

        return Err(msg.into());
    }

    sqlx::query!("DELETE FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    let msg = if user.is_some() {
        format!("Removed {} from the birthday list!", user.unwrap().name)
    } else {
        format!("Removed user from the birthday list with id: {}!", user_id)
    };
    ctx.say(msg).await?;

    Ok(())
}

/// Edit a user's birthday
#[poise::command(
    slash_command,
    check = "bday_channel_check",
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn edit (
    ctx: Context<'_>,
    #[description = "User to edit information."] user: serenity::User,
    #[description = "Birthmonth."]
    #[min = 1_u8]
    #[max = 12_u8] month: Option<u8>,
    #[description = "Birthday."]
    #[min = 1_u8]
    #[max = 31_u8] day: Option<u8>,
    #[description = "A nickname for the user."]
    #[max_length = 30] nickname: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let user_id = user.id.get();

    // Validate information
    let birthday_info = sqlx::query_as!(Birthday, "SELECT * FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_optional(&ctx.data().database)
        .await
        .unwrap();

    if birthday_info.is_none() {
        return Err(format!("{}'s birthday is not saved!", user.name).into());
    }

    if month.is_none() && day.is_none() && nickname.is_none() {
        return Err("You must choose at least one option to edit!".into());
    }

    // Build and execute query
    let birthmonth = month.unwrap_or(birthday_info.as_ref().unwrap().birthmonth);
    let birthday = day.unwrap_or(birthday_info.as_ref().unwrap().birthday);
    let nickname = nickname.unwrap_or(birthday_info.unwrap().nickname.unwrap_or(String::new()));

    sqlx::query!("UPDATE birthday SET birthmonth = ?, birthday = ?, nickname = ? WHERE guild_id = ? AND user_id = ?", birthmonth, birthday, nickname, guild_id, user_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("{}, altered information for {}!", ctx.author(), user.name)).await?;

    Ok(())
}

/// Get a user's birthday
#[poise::command(
    slash_command,
    check = "bday_channel_check",
)]
pub async fn info(
    ctx: Context<'_>,
    #[description = "User to check."] user: serenity::User
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let user_id = user.id.get();
    let count = sqlx::query!("SELECT COUNT(user_id) AS count FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    if count.count == 0 {
        return Err(format!("{}'s birthday is not saved!", user.name).into());
    }

    let birthday = sqlx::query_as!(Birthday, "SELECT * FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    let formatted_day = match birthday.birthday {
        1 => "1st".to_string(),
        2 => "2nd".to_string(),
        3 => "3rd".to_string(),
        21 => "21st".to_string(),
        22 => "22nd".to_string(),
        23 => "23rd".to_string(),
        31 => "31st".to_string(),
        _ => format!("{}th", birthday.birthday)
    };

    let nickname = if birthday.nickname.is_some() {
        format!(" ({})", birthday.nickname.unwrap())
    } else {
        String::new()
    };
    let msg = format!("{}{}'s birthday is on {} {}!", user.name, nickname, MONTHS[birthday.birthmonth as usize - 1], formatted_day);

    ctx.say(msg).await?;

    Ok(())
}

/// Set the channel to send birthday announcements
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS"
)]
pub async fn setchannel(
    ctx: Context<'_>,
    #[description = "Channel to post announcements in."] channel: serenity::Channel
) -> Result<(), Error> {
    let channel_id = channel.id().get();
    let guild_id = ctx.guild_id().unwrap().get();

    sqlx::query!("UPDATE guild_settings SET birthday_channel = ? WHERE guild_id = ?", channel_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Now posting birthday announcements in {}!", channel)).await?;

    Ok(())
}

/// Set the role to give the birthday person
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "bday_channel_check"
)]
pub async fn setrole(
    ctx: Context<'_>,
    #[description = "Role to give the birthday person."] role: Option<serenity::Role>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Build/run query
    let role_id: Option<u64> = role.as_ref().map(|r| r.id.get());

    sqlx::query!("UPDATE guild_settings SET birthday_role = ? WHERE guild_id = ?", role_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    // Message author
    if role.is_none() {
        ctx.say(format!("{}, no longer giving a role on a user's birthday!", ctx.author())).await?;
    } else {
        ctx.say(format!("{}. now giving the {} role on birthdays!", ctx.author(), role.unwrap().name)).await?;
    }

    Ok(())
}

/// List all birthdays on the server
#[poise::command(
    slash_command,
    required_permissions = "MANAGE_CHANNELS",
    check = "bday_channel_check",
)]
pub async fn list(
    ctx: Context<'_>,

    #[min = 1_u8]
    #[max = 12_u8] 
    #[description = "Optionally only list birthdays for a specific month."] month: Option<u8>
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();

    // Fetch any birthdays if any
    let month_query = if month.is_some() {
        format!(" AND birthmonth = {}", month.unwrap())
    } else {
        String::new()
    };

    let query = format!("SELECT * FROM birthday WHERE guild_id = {guild_id}{month_query} ORDER BY birthmonth, birthday");

    let birthdays: Vec<Birthday> = sqlx::query_as(&query)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();
    
    if birthdays.is_empty() {
        return Err("No birthdays found for this server!".into());
    }

    // Build birthday list embed
    let mut birthday_embed = serenity::CreateEmbed::new()
        .colour(0xFFC60A)
        .title("User Birthdays");

    let mut embed_desc = String::new();

    for (i, birthday) in birthdays.into_iter().enumerate() {
        // Fetch username from UserID
        let username = serenity::UserId::new(birthday.user_id).to_user(&ctx.serenity_context().http).await.unwrap().name;

        embed_desc = format!("{embed_desc}\n- **[{}]** {} - ID: {} - Birthday: {}/{}",
            i + 1,
            username,
            birthday.user_id,
            birthday.birthday,
            birthday.birthmonth);
    }
    birthday_embed = birthday_embed.description(embed_desc);

    // Send birthday list
    ctx.send(poise::CreateReply::default()
        .embed(birthday_embed)).await?;

    Ok(())
}

//--------------------------
// Birthday Checker
//--------------------------
pub async fn birthday_check(ctx: &serenity::Context, data: &Data) {
    // Check the time (10 UTC, 2 Pacific)
    let current_time = Utc::now();

    if current_time.format("%H:%M:%S").to_string() == "10:00:00" {
        let registered_guild_channels = sqlx::query!("SELECT guild_id, birthday_channel, birthday_role FROM guild_settings")
            .fetch_all(&data.database)
            .await
            .unwrap();

        let current_date = Utc::now().format("%m-%d").to_string();
        let current_date: Vec<u8> = current_date.split('-').map(|i| i.parse::<u8>().unwrap()).collect();

        // Loop the registered guilds
        for guild in registered_guild_channels {
            // Check for birthday channel; If none, skip everything
            if guild.birthday_channel.is_none() {
                continue;
            }
            let channel_id = serenity::ChannelId::new(guild.birthday_channel.unwrap());

            let guild_birthdays = sqlx::query!("SELECT * FROM birthday WHERE guild_id = ?", guild.guild_id)
                .fetch_all(&data.database)
                .await
                .unwrap();

            let birthday_gifs = grab_custom_gifs(&data.database, GIFType::Birthday, guild.guild_id, GIFDBQueryType::Normal).await;

            for birthday in guild_birthdays {
                if birthday.birthmonth == current_date[0] && birthday.birthday == current_date[1] {
                    let username = determine_username(ctx, &data.database, birthday.nickname, birthday.guild_id, birthday.user_id).await;

                    // Take care of the birthday message
                    let bday_msg = format!("Happy birthday, {username}! :birthday: We hope you have a great day!");

                    let mut embed = serenity::CreateEmbed::new()
                        .colour(0xFF0095)
                        .thumbnail("https://media.istockphoto.com/vectors/birthday-cake-vector-isolated-vector-id901911608?k=6&m=901911608&s=612x612&w=0&h=d6v27h_mYUaUe0iSrtoX5fTw-2wGVIY4UTbQPeI-T5k=")
                        .title(bday_msg);

                    let mut msg = serenity::CreateMessage::new()
                        .content("@everyone :birthday:");

                    // Add GIF if one is possible to the message
                    let random_gif = if !birthday_gifs.is_empty() {
                        let mut rng = thread_rng();
                        Some(&birthday_gifs[rng.gen_range(0..birthday_gifs.len())].filename)
                    } else {
                        None
                    };
                        
                    if let Some(filename) = random_gif {
                        let path = format!("CustomGIFs/{}/birthday/{filename}.gif", birthday.guild_id);
                        let gif = serenity::CreateAttachment::path(path).await.unwrap();
                    
                        embed = embed.image(format!("attachment://{filename}.gif"));
                        msg = msg.add_file(gif);
                    }
                    msg = msg.embed(embed);

                    channel_id.send_message(&ctx, msg).await.unwrap();

                    // Give birthday role
                    if guild.birthday_role.is_some() {
                        let birthday_guild = serenity::GuildId::new(guild.guild_id);
                        let birthday_member = birthday_guild.member(&ctx, serenity::UserId::new(birthday.user_id)).await.unwrap();

                        birthday_member.add_role(&ctx, serenity::RoleId::new(guild.birthday_role.unwrap())).await.unwrap();
                    }
                } else {
                    // Remove birthday role if member has it
                    if guild.birthday_role.is_some() {
                        let birthday_guild = serenity::GuildId::new(guild.guild_id);
                        let birthday_member = birthday_guild.member(&ctx, serenity::UserId::new(birthday.user_id)).await;

                        match birthday_member {
                            Ok(m) => m.remove_role(&ctx, serenity::RoleId::new(guild.birthday_role.unwrap())).await.unwrap(),
                            Err(e) => println!("{:?}", e)
                        }
                    }
                }
            }
        }
    }
}