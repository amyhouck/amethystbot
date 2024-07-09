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
            Ok(_) => println!("[ LOG ] New user added to database - GuildID: {guild_id} - UserID: {user_id}"),
            Err(e) => {
                println!("[ ERROR ] Error occurred while adding a user to the database:");
                println!("{e}");
            }
        }
    }
}