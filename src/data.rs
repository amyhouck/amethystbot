use crate::Context;
use poise::serenity_prelude as serenity;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use std::time::Duration;
use tracing::{info, error};

//--------------------------
// Handler for the bot data
//--------------------------
#[derive(Debug, Clone)]
pub struct Data { // User data, which is stored and accessible in all command invocations
    pub database: sqlx::MySqlPool,
    pub client: reqwest::Client,
}

impl Data {
    pub async fn init() -> Self {
        // Set Data
        let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
        let database = sqlx::mysql::MySqlPool::connect(&database_url).await.unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("AmethystBot/1.0"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

        let req_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .unwrap();

        Data {
            database,
            client: req_client,
        }
    }
}

//--------------------------
// User table checker
//--------------------------
pub async fn user_table_check(ctx: Context<'_>, user: &serenity::User) {
    let guild_id = ctx.guild_id().unwrap();

    // Grab user info
    let db_user = sqlx::query!("SELECT display_name, COUNT(user_id) AS count FROM users WHERE guild_id = ? AND user_id = ?", guild_id.get(), user.id.get())
            .fetch_one(&ctx.data().database)
            .await
            .unwrap();

    // If user doesn't exist, add them. Returns after adding
    if db_user.count == 0 {
        user_table_add(&ctx.data().database, guild_id.get(), user.id.get(), user.display_name().to_string()).await;
    }
}

//--------------------------
// Add user to database
//--------------------------
pub async fn user_table_add(database: &sqlx::MySqlPool, guild_id: u64, user_id: u64, display_name: String) {
    let users_table_query_attempt = sqlx::query!("INSERT INTO users (guild_id, user_id, display_name) VALUES (?, ?, ?)", guild_id, user_id, display_name)
        .execute(database)
        .await;

    match users_table_query_attempt {
        Ok(_) => info!("[ USER ] New user added to stats table - Guild ID: {guild_id} - User ID: {user_id}"),
        Err(e) => error!("[ USER ] An error occurred adding user to stats table: {e}")
    }
    
    let user_settings_query = sqlx::query!("INSERT INTO user_settings (guild_id, user_id) VALUES (?, ?)", guild_id, user_id)
        .execute(database)
        .await;
        
    match user_settings_query {
        Ok(_) => info!("[ USER ] New user added to user_settings table - Guild ID: {guild_id} - User ID: {user_id}"),
        Err(e) => error!("[ USER ] An error occurred adding user to user_settings table: {e}")
    }
}

//--------------------------
// Alter display name in DB if needed
//--------------------------
pub async fn alter_db_display_name(
    database: &sqlx::MySqlPool,
    guild_id: u64,
    user_id: u64,
    display_name: String
) {
    let query = sqlx::query!("UPDATE users SET display_name = ? WHERE guild_id = ? AND user_id = ? AND display_name != ?", display_name, guild_id, user_id, display_name)
            .execute(database)
            .await
            .unwrap();
    
    if query.rows_affected() == 1 {
        info!("[ USER ] Changed user display name in user stats table - Guild ID: {guild_id} - User ID: {user_id}");
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
    pub vctrack_monthly_time: u32,
    pub display_name: String,
    pub rps_win: u32,
    pub rps_loss: u32,
    pub rps_tie: u32,
    pub roulette_deaths: u32,
    pub hug_sent: u32,
    pub hug_received: u32,
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
    pub roulette_rounds: u32,
    pub rps_rounds: u32,
    pub hug_sent: u32,
}
