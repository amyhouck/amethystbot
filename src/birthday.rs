use crate::{Context, Error};
use poise::serenity_prelude as serenity;

//--------------------
// Data
//--------------------
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
async fn bday_channel_check(ctx: Context<'_>) -> Result<bool, Error> {
    let guild_id = *ctx.guild_id().unwrap().as_u64();

    let settings = sqlx::query!("SELECT birthday_channel FROM guild_settings WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    match settings.birthday_channel {
        Some(_) => Ok(true),
        None => {
            ctx.send(|m| m
                .content("You must select a channel to post birthday announcements in before using this command!")
                .ephemeral(true)
            ).await?;
            Ok(false)
        }
    }
}

//--------------------
// Commands
//--------------------
#[poise::command(
    slash_command,
    guild_only,
    subcommands("add", "remove", "edit", "setchannel", "info")
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
        guild_id: *ctx.guild_id().unwrap().as_u64(),
        user_id: *user.id.as_u64(),
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

    ctx.send(|m| {
        m.content(format!("Successfully added {}'s birthday!", user.name));
        m.ephemeral(false)
    }).await?;

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
    let guild_id = *ctx.guild_id().unwrap().as_u64();

    let user_id = match &user {
        Some(u) => *u.id.as_u64(),
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
    let guild_id = *ctx.guild_id().unwrap().as_u64();
    let user_id = *user.id.as_u64();

    // Validate information
    let birthday_info = sqlx::query_as!(Birthday, "SELECT * FROM birthday WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_optional(&ctx.data().database)
        .await
        .unwrap();

    if birthday_info.is_none() {
        return Err(format!("{}'s birthday is not saved!", user.name).into());
    }

    if month.is_none() && day.is_none() && nickname.is_none() {
        return Err(format!("You must choose at least one option to edit!").into());
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
    let guild_id = *ctx.guild_id().unwrap().as_u64();
    let user_id = *user.id.as_u64();
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
    let channel_id = *channel.id().as_u64();
    let guild_id = *ctx.guild_id().unwrap().as_u64();

    sqlx::query!("UPDATE guild_settings SET birthday_channel = ? WHERE guild_id = ?", channel_id, guild_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();

    ctx.say(format!("Now posting birthday announcements in {}!", channel)).await?;

    Ok(())
}