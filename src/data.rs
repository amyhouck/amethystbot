use crate::log;

//--------------------------
// User table checker
//--------------------------
pub async fn user_table_check(database: &sqlx::MySqlPool, guild_id: u64, user_id: u64) {
    // If user doesn't exist, add row
    let count = sqlx::query!("SELECT COUNT(user_id) AS count FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .fetch_one(database)
            .await
            .unwrap();

    if count.count == 0 {
        let query_attempt = sqlx::query!("INSERT INTO users (guild_id, user_id) VALUES (?, ?)", guild_id, user_id)
            .execute(database)
            .await;
        
        match query_attempt {
            Ok(_) => log::write_log(log::LogType::UserDBRegister { guild_id, user_id }),
            Err(e) => log::write_log(log::LogType::DBError { db_error: e.to_string() })
        }
    }
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