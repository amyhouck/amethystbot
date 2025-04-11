mod data;
mod modules;
mod events;

use data::{alter_db_display_name, determine_display_username, user_table_add, user_table_check, Data};
use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use chrono::Utc;
use cron::Schedule;
use std::{str::FromStr, sync::Arc};
use modules::*;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn listener(ctx: &serenity::Context, event: &serenity::FullEvent, _framework: poise::FrameworkContext<'_, Data, Error>, data: &Data) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { .. } => {
            log::write_log(log::LogType::BotStartup);
            let ctx = Arc::new(ctx.clone());
            let data = Arc::new(data.clone());

            // Birthday loop
            let new_ctx = Arc::clone(&ctx);
            let new_data = Arc::clone(&data);
            tokio::spawn(async move {
                loop {
                    birthday::birthday_check(&new_ctx, &new_data).await;

                    // Calculate sleep until the next proper birthday time
                    let current_time = chrono::Utc::now();
                    let expression = "0 0 10 * * * *";
                    let schedule = Schedule::from_str(expression).unwrap();
                    let schedule: Vec<_> = schedule.upcoming(Utc).take(1).collect();
                    let duration = schedule[0].signed_duration_since(current_time);
                    
                    log::write_log(log::LogType::BirthdayTimerReset { duration: duration.to_string() });

                    tokio::time::sleep(duration.to_std().unwrap()).await;
                }
            });

            // VCTracker Monthly Reset
            let new_data = Arc::clone(&data);
            tokio::spawn(async move {
                loop {
                    vctracker::vctracker_reset_monthly(&new_data.database).await;

                    // Calculate sleep to 1st of next month
                    let current_time = chrono::Utc::now();
                    let expression = "0 0 0 1 * * *";
                    let schedule = Schedule::from_str(expression).unwrap();
                    let schedule: Vec<_> = schedule.upcoming(Utc).take(1).collect();
                    let duration = schedule[0].signed_duration_since(current_time);

                    log::write_log(log::LogType::VCTrackerResetMonthlyDuration { duration: duration.to_string() });

                    tokio::time::sleep(duration.to_std().unwrap()).await;
                }
            });
        },

        serenity::FullEvent::GuildCreate { guild, is_new } => {
            let guild_id = guild.id.get();
            let is_new = is_new.unwrap_or(false);

            // Add guild to settings tables if new to the bot
            if is_new {
                let query = format!("
                    INSERT IGNORE INTO guild_settings (guild_id) VALUES ({guild_id});
                    INSERT IGNORE INTO welcome (guild_id) VALUES ({guild_id})
                ");

                sqlx::raw_sql(&query)
                    .execute(&data.database)
                    .await
                    .unwrap();

                log::write_log(log::LogType::BotGuildDBRegister { guild_id, table_name: String::from("welcome") });
                log::write_log(log::LogType::BotGuildDBRegister { guild_id, table_name: String::from("guild_settings") });
            }

            log::write_log(log::LogType::BotGuildLogin { guild_id });

            // VC tracking safeguard for disconnect users
            let guild_voice_states = guild.voice_states.clone();
            let user_join_times = sqlx::query!("SELECT user_id, vctrack_join_time FROM users WHERE guild_id = ? AND vctrack_join_time != 0", guild_id)
                .fetch_all(&data.database)
                .await
                .unwrap();

            for user in user_join_times {
                match guild_voice_states.get(&serenity::UserId::new(user.user_id)) {
                    Some(_) => continue,
                    None => {
                        sqlx::query!("UPDATE users SET vctrack_join_time = 0 WHERE guild_id = ? AND user_id = ?", guild_id, user.user_id)
                            .execute(&data.database)
                            .await
                            .unwrap();

                        log::write_log(log::LogType::VCTrackerSafeguardAdjustment { guild_id, user_id: user.user_id });
                    }
                }
            }
        },

        serenity::FullEvent::GuildMemberAddition { new_member } => {
            let guild_id = new_member.guild_id.get();
            log::write_log(log::LogType::WelcomeNewUser { guild_id });

            // Grab all info and check for channel
            let welcome = sqlx::query!("SELECT * FROM welcome WHERE guild_id = ?", guild_id)
                .fetch_one(&data.database)
                .await
                .unwrap();

            if welcome.channel_id.is_none() { return Ok(()); }

            // Build welcome embed then post
            let welcome_embed = serenity::CreateEmbed::new()
                .colour(0xC0C0C0)
                .thumbnail(new_member.face())
                .image(welcome.image_url.unwrap_or(String::new()))
                .description(welcome.message.unwrap_or(String::new()))
                .title(format!("Welcome, {}!", new_member.display_name()));

            let channel = serenity::ChannelId::new(welcome.channel_id.unwrap());

            channel.send_message(&ctx, serenity::CreateMessage::new().embed(welcome_embed)).await.unwrap();

            user_table_add(&data.database, guild_id, new_member.user.id.get(), new_member.display_name().to_string()).await;
        },

        serenity::FullEvent::GuildMemberRemoval { guild_id, user, ..} => {
            // Remove member from database records
            let guild_id = guild_id.get();
            let user_id = user.id.get();

            // Remove user from birthday, users
            let query = format!("
                DELETE FROM birthday WHERE guild_id = {guild_id} AND user_id = {user_id};
                DELETE FROM users WHERE guild_id = {guild_id} AND user_id = {user_id};
                DELETE FROM user_settings WHERE guild_id = {guild_id} AND user_id = {user_id}
            ");

            sqlx::raw_sql(&query)
                .execute(&data.database)
                .await
                .unwrap();

            log::write_log(log::LogType::UserDBRemove);

            // Send leave message if channel is set in server settings
            let leave_channel_id = sqlx::query!("SELECT member_leave_channel_id FROM guild_settings WHERE guild_id = ?", guild_id)
                .fetch_one(&data.database)
                .await
                .unwrap()
                .member_leave_channel_id;

            if leave_channel_id.is_some() {
                let channel_id = serenity::ChannelId::new(leave_channel_id.unwrap());
                let msg = serenity::CreateMessage::new().content(format!("***{} has left the server.***", user.name));

                channel_id.send_message(&ctx, msg).await.unwrap();
            }
        },

        serenity::FullEvent::GuildMemberUpdate { new, .. } => {
            let new = match new {
                Some(data) => data,
                None => return Ok(())
            };

            let display_name = determine_display_username(&ctx.http, &new.user, new.guild_id).await;
            alter_db_display_name(&data.database, new.guild_id.get(), new.user.id.get(), display_name).await;
        },

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.id.get() == 375805687529209857u64 && new_message.content.contains("Desiner") {
                new_message.channel_id.say(&ctx, "Desiner").await?;
            }
        },

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
            // Check for user in the database
            // We do it here manually since the data::user_table_check function requires poise::Context and not serenity::Content.
            // No point in writing a second check function using serenity::Context if it's only going to be used here for now
            let guild_id = new.guild_id.unwrap();
            let user_id = new.user_id;

            let db_user = sqlx::query!("SELECT display_name, COUNT(user_id) AS count FROM users WHERE guild_id = ? AND user_id = ?", guild_id.get(), user_id.get())
                .fetch_one(&data.database)
                .await
                .unwrap();

            let display_name = determine_display_username(&ctx.http, &new.member.as_ref().unwrap().user, guild_id).await;

            // If user doesn't exist, add them. Returns after adding
            if db_user.count == 0 {
                user_table_add(&data.database, guild_id.get(), user_id.get(), display_name).await;
            }

            // Events here
            // Handle connection to VC
            if old.is_none() && new.channel_id.is_some() {
                events::on_user_vc_connect(data, old, new).await?;
            }
            
            // Handle disconnection from VC
            if old.is_some() && new.channel_id.is_none() {
                events::on_user_vc_disconnect(data, old, new).await?;
            }

            // Handle channel movement
            if old.is_some() && new.channel_id.is_some() {
                events::on_user_vc_move(data, old, new).await?;
            }
        },
        _ => {}
    }

    Ok(())
}

//--------------------------
// Main
//--------------------------
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Build bot
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MEMBERS | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data::init().await)
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![
                birthday::bday(),
                misc::slap(),
                misc::cookie(),
                misc::tea(),
                misc::cake(),
                welcome::welcome(),
                welcome::setleavechannel(),
                mtg::mtg(),
                stats::stats(),
                stats::serverstats(),
                vctracker::vctracker(),
                vctracker::vctop(),
                quotes::addquote(),
                quotes::quote(),
                quotes::delquote(),
                quotes::setquoterole(),
                quotes::listquotes(),
                customgifs::addgif(),
                customgifs::delgif(),
                customgifs::listgifs(),
                customgifs::setgifrole(),
                settings::settings(),

                minigames::bomb::bomb(),
                minigames::rockpaperscisso::rps(),
                minigames::roulette::roulette(),
            ],
            pre_command: |ctx| {
                Box::pin(async move {
                    log::write_log(log::LogType::CommandExecution { ctx });

                    user_table_check(ctx, ctx.author()).await;
                })
            },
            event_handler: |ctx, event, framework, data| Box::pin(listener(ctx, event, framework, data)),
            ..Default::default()
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();

        log::write_log(log::LogType::BotShutdown);
        shard_manager.shutdown_all().await;
    });
    
    client.start().await.unwrap();
}