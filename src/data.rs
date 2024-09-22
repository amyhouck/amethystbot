use crate::log;
use poise::serenity_prelude as serenity;

//--------------------------
// User table checker
//--------------------------
pub async fn user_table_check(database: &sqlx::MySqlPool, http: &serenity::Http, guild_id: serenity::GuildId, user: &serenity::User) {
    // Grab user info
    let db_user = sqlx::query!("SELECT display_name, COUNT(user_id) AS count FROM users WHERE guild_id = ? AND user_id = ?", guild_id.get(), user.id.get())
            .fetch_one(database)
            .await
            .unwrap();

    let display_name = determine_display_username(http, user, guild_id).await;

    // If user doesn't exist, add them. Returns after adding
    if db_user.count == 0 {
        let query_attempt = sqlx::query!("INSERT INTO users (guild_id, user_id, display_name) VALUES (?, ?, ?)", guild_id.get(), user.id.get(), display_name)
            .execute(database)
            .await;
        
        match query_attempt {
            Ok(_) => log::write_log(log::LogType::UserDBRegister { guild_id: guild_id.get(), user_id: user.id.get() }),
            Err(e) => log::write_log(log::LogType::DBError { db_error: e.to_string() })
        }

        return;
    }

    // If the display names don't match, update database
    if db_user.display_name.unwrap() != display_name {
        sqlx::query!("UPDATE users SET display_name = ? WHERE guild_id = ? AND user_id = ?", display_name, guild_id.get(), user.id.get())
            .execute(database)
            .await
            .unwrap();

        log::write_log(log::LogType::UserDBNameChange { guild_id: guild_id.get(), user_id: user.id.get() });
    }
}

//--------------------------
// Determine username to display
//--------------------------
pub async fn determine_display_username(
    http: &serenity::Http,
    user: &serenity::User,
    guild_id: serenity::GuildId
) -> String {
    let nick = user.nick_in(http, guild_id).await;

    nick.unwrap_or_else(||
        match &user.global_name {
            Some(n) => n.to_string(),
            None => user.name.to_string()
        }
    )
}

//--------------------------
// User structure
//--------------------------
#[derive(Debug)]
#[allow(dead_code)]
pub struct User {
    pub guild_id: u64,
    pub user_id: u64,
    pub cookie_sent: u32,
    pub cookie_received: u32,
    pub slap_sent: u32,
    pub slap_received: u32,
    pub cake_sent: u32,
    pub cake_received: u32,
    pub cake_glados: u32,
    pub tea_sent: u32,
    pub tea_received: u32,
    pub bomb_sent: u32,
    pub bomb_defused: u32,
    pub bomb_failed: u32,
    pub vctrack_join_time: u32,
    pub vctrack_total_time: u32,
    pub vctrack_monthly_time: u32,
    pub display_name: String,
}

//--------------------------
// Server stats structure
//--------------------------
#[derive(Default)]
pub struct ServerStats {
    pub cookie_sent: u32,
    pub slap_sent: u32,
    pub cake_sent: u32,
    pub tea_sent: u32,
    pub bomb_sent: u32,
    pub bomb_defused: u32,
    pub bomb_failed: u32,
    pub glados_appearances: u32,
}