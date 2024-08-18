#![allow(unused_variables)]

use poise::serenity_prelude as serenity;
use serenity::VoiceState;
use crate::{Data, Error};

// "Custom" Event Calls

// VoiceStateUpdate Calls
// If user connects to VC
pub async fn on_user_vc_connect(data: &Data, old: &Option<VoiceState>, new: &VoiceState) -> Result<(), Error> {
    // VCTracker Module
    // Get needed info
    let guild_id = new.guild_id.unwrap().get();
    let user_id = new.user_id.get();
    
    crate::data::user_table_check(&data.database, guild_id, user_id).await;

    // Set ignored channel id
    let ignored_channel_id = sqlx::query!("SELECT vctrack_ignored_channel FROM guild_settings WHERE guild_id = ?", guild_id)
            .fetch_one(&data.database)
            .await
            .unwrap();
    let ignored_channel_id = ignored_channel_id.vctrack_ignored_channel.unwrap_or(0);

    // Check if new.channel_id isn't ignored channel
    if new.channel_id.unwrap().get() != ignored_channel_id {
        sqlx::query!("UPDATE users SET vctrack_join_time = UNIX_TIMESTAMP() WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .execute(&data.database)
            .await
            .unwrap();
    }

    Ok(())
}

// If user disconnects from VC
pub async fn on_user_vc_disconnect(data: &Data, old: &Option<VoiceState>, new: &VoiceState) -> Result<(), Error> {
    // VCTracker Module
    // Get needed info
    let guild_id = new.guild_id.unwrap().get();
    let user_id = new.user_id.get();
    
    crate::data::user_table_check(&data.database, guild_id, user_id).await;

    // Set ignored channel id
    let ignored_channel_id = sqlx::query!("SELECT vctrack_ignored_channel FROM guild_settings WHERE guild_id = ?", guild_id)
            .fetch_one(&data.database)
            .await
            .unwrap();
    let ignored_channel_id = ignored_channel_id.vctrack_ignored_channel.unwrap_or(0);

    // Check if user join time is 0
    let join_time = sqlx::query!("SELECT vctrack_join_time FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
        .fetch_one(&data.database)
        .await
        .unwrap();

    if join_time.vctrack_join_time == 0 { return Ok(()); }

    // Check if old.channel_id isn't ignored channel
    if old.as_ref().unwrap().channel_id.unwrap().get() != ignored_channel_id {
    let query = format!("
        UPDATE users SET vctrack_total_time = vctrack_total_time + (UNIX_TIMESTAMP() - vctrack_join_time) WHERE guild_id = {guild_id} AND user_id = {user_id};
        UPDATE users SET vctrack_join_time = 0 WHERE guild_id = {guild_id} AND user_id = {user_id}");

    sqlx::raw_sql(&query)
        .execute(&data.database)
        .await
        .unwrap();
    }

    Ok(())
}

// If user moves between VC channels
pub async fn on_user_vc_move(data: &Data, old: &Option<VoiceState>, new: &VoiceState) -> Result<(), Error> {
    // VCTracker Module
    // Get needed info
    let guild_id = new.guild_id.unwrap().get();
    let user_id = new.user_id.get();
    
    crate::data::user_table_check(&data.database, guild_id, user_id).await;

    // Set ignored channel id
    let ignored_channel_id = sqlx::query!("SELECT vctrack_ignored_channel FROM guild_settings WHERE guild_id = ?", guild_id)
            .fetch_one(&data.database)
            .await
            .unwrap();
    let ignored_channel_id = ignored_channel_id.vctrack_ignored_channel.unwrap_or(0);
    
    let old_cid = old.as_ref().unwrap().channel_id.unwrap().get();
    let new_cid = new.channel_id.unwrap().get();

    // Check if new.channel_id is ignored channel
    // - If so, act as if disconnecting
    // - If not, act as if connecting
    if old_cid != new_cid && new_cid == ignored_channel_id {
        // Check if user join time is 0
        let join_time = sqlx::query!("SELECT vctrack_join_time FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .fetch_one(&data.database)
            .await
            .unwrap();

        if join_time.vctrack_join_time == 0 { return Ok(()); }

        let query = format!("
            UPDATE users SET vctrack_total_time = vctrack_total_time + (UNIX_TIMESTAMP() - vctrack_join_time) WHERE guild_id = {guild_id} AND user_id = {user_id};
            UPDATE users SET vctrack_join_time = 0 WHERE guild_id = {guild_id} AND user_id = {user_id}");

        sqlx::raw_sql(&query)
            .execute(&data.database)
            .await
            .unwrap();

        return Ok(());
    }

    if old_cid != new_cid && old_cid == ignored_channel_id {
        sqlx::query!("UPDATE users SET vctrack_join_time = UNIX_TIMESTAMP() WHERE guild_id = ? AND user_id = ?", guild_id, user_id)
            .execute(&data.database)
            .await
            .unwrap();
    }

    Ok(())
}