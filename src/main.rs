mod data;
mod modules;
mod events;

use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use chrono::Utc;
use cron::Schedule;
use rand::{Rng, thread_rng};
use std::{str::FromStr, sync::Arc};
use std::time::Duration;
use modules::*;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};


#[derive(Debug, Clone)]
pub struct Data { // User data, which is stored and accessible in all command invocations
    database: sqlx::MySqlPool,
    birthday_gifs: Vec<String>,
    slap_gifs: Vec<String>,
    self_slap_gifs: Vec<String>,
    tea_gifs: Vec<String>,
    cake_gifs: Vec<String>,
    client: reqwest::Client,
}

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
                    birthday_check(&new_ctx, &new_data).await;

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
        },

        serenity::FullEvent::GuildMemberRemoval { guild_id, user, ..} => {
            let guild_id = guild_id.get();
            let user_id = user.id.get();

            // Remove user from birthday, users
            let query = format!("
                DELETE FROM birthday WHERE guild_id = {guild_id} AND user_id = {user_id};
                DELETE FROM users WHERE guild_id = {guild_id} AND user_id = {user_id}
            ");

            sqlx::raw_sql(&query)
                .execute(&data.database)
                .await
                .unwrap();

            log::write_log(log::LogType::UserDBRemove);
        },

        serenity::FullEvent::Message { new_message } => {
            if new_message.author.id.get() == 375805687529209857u64 && new_message.content.contains("Desiner") {
                new_message.channel_id.say(&ctx, "Desiner").await?;
            }
        },

        serenity::FullEvent::VoiceStateUpdate { old, new } => {
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
// Birthday Checker
//--------------------------
async fn birthday_check(ctx: &serenity::Context, data: &Data) {
    // Check the time (10 UTC, 2 Pacific)
    let current_time = Utc::now();

    if current_time.format("%H:%M").to_string() == String::from("10:00") {
        let registered_guild_channels = sqlx::query!("SELECT guild_id, birthday_channel, birthday_role FROM guild_settings")
            .fetch_all(&data.database)
            .await
            .unwrap();

        let current_date = Utc::now().format("%m-%d").to_string();
        let current_date: Vec<u8> = current_date.split('-').map(|i| i.parse::<u8>().unwrap()).collect();

        // Loop the registered guilds
        for guild in registered_guild_channels {
            // Check for birthday channel
            if guild.birthday_channel.is_none() {
                continue;
            }
            let channel_id = serenity::ChannelId::new(guild.birthday_channel.unwrap());

            // Select birthdays
            let guild_birthdays = sqlx::query!("SELECT * FROM birthday WHERE guild_id = ?", guild.guild_id)
                .fetch_all(&data.database)
                .await
                .unwrap();

            for birthday in guild_birthdays {
                if birthday.birthmonth == current_date[0] && birthday.birthday == current_date[1] {
                    // Take care of the birthday message
                    let username = birthday.nickname.unwrap_or(serenity::UserId::new(birthday.user_id).to_user(&ctx).await.unwrap().name);
                    let bday_msg = format!("Happy birthday, {username}! :birthday: We hope you have a great day!");

                    let random_gif = {
                        let mut rng = thread_rng();
                        rng.gen_range(0..data.birthday_gifs.len())
                    };

                    let embed = serenity::CreateEmbed::new()
                        .colour(0xFF0095)
                        .thumbnail("https://media.istockphoto.com/vectors/birthday-cake-vector-isolated-vector-id901911608?k=6&m=901911608&s=612x612&w=0&h=d6v27h_mYUaUe0iSrtoX5fTw-2wGVIY4UTbQPeI-T5k=")
                        .title(bday_msg)
                        .image(&data.birthday_gifs[random_gif]);

                    let msg = serenity::CreateMessage::new()
                        .content("@everyone :birthday:")
                        .embed(embed);

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
                        let birthday_member = birthday_guild.member(&ctx, serenity::UserId::new(birthday.user_id)).await.unwrap();

                        birthday_member.remove_role(&ctx, serenity::RoleId::new(guild.birthday_role.unwrap())).await.unwrap();
                    }
                }
            }
        }
    }
}

//--------------------------
// Main
//--------------------------
#[tokio::main]
async fn main() {
    dotenv().ok();

    // Set Data
    let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let database = sqlx::mysql::MySqlPool::connect(&database_url).await.unwrap();
    sqlx::migrate!("./migrations").run(&database).await.unwrap();

    let birthday_gifs: Vec<String> = vec![
        "https://media.giphy.com/media/WRL7YgP42OKns22wRD/giphy.gif".to_string(),
        "https://media.giphy.com/media/g5R9dok94mrIvplmZd/giphy.gif".to_string(),
        "https://media.giphy.com/media/l4KhS0BOFBhU2SYIU/giphy.gif".to_string(),
        "https://media.giphy.com/media/l4KibWpBGWchSqCRy/giphy.gif".to_string(),
        "https://media.giphy.com/media/arGdCUFTYzs2c/giphy.gif".to_string(),
    ];

    let slap_gifs: Vec<String> = vec![
        "https://media.tenor.com/7_ktpmstpIkAAAAC/troutslap.gif".to_string(),
        "https://media.tenor.com/w5wm0GtfI9EAAAAd/tenor.gif".to_string(),
    ];

    let self_slap_gifs: Vec<String> = vec![
        "https://i.makeagif.com/media/6-19-2015/rh-Yg3.gif".to_string(),
    ];

    let tea_gifs: Vec<String> = vec![
        "https://media1.tenor.com/m/gyNQ_0VaG-0AAAAC/dalek-exterminate.gif".to_string(),
        "https://media1.tenor.com/m/IXyaShXuq_IAAAAC/doctor-who-sip.gif".to_string(),
    ];

    let cake_gifs: Vec<String> = vec![
        "https://media1.tenor.com/m/Y0RcGnmG2DkAAAAC/cake-birthday-cake.gif".to_string(),
        "https://media1.tenor.com/m/uhzaWzEXdjcAAAAd/cake-sprinkles.gif".to_string(),
    ];

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("AmethystBot/1.0"));
    headers.insert(ACCEPT, HeaderValue::from_static("*/*"));

    let req_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()
        .unwrap();

    let data = Data {
        database,
        birthday_gifs,
        slap_gifs,
        self_slap_gifs,
        tea_gifs,
        cake_gifs,
        client: req_client,
    };

    // Build bot
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::GUILD_MEMBERS | serenity::GatewayIntents::GUILD_MESSAGES | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_VOICE_STATES;

    let framework = poise::Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(data.clone())
            })
        })
        .options(poise::FrameworkOptions {
            commands: vec![
                birthday::bday(),
                misc::slap(),
                misc::cookie(),
                misc::bomb(),
                misc::tea(),
                misc::cake(),
                welcome::welcome(),
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
                bot_management::shutdown(),
                bot_management::reset_display_names(),
            ],
            pre_command: |ctx| {
                Box::pin(async move {
                    log::write_log(log::LogType::CommandExecution { ctx });
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

    client.start().await.unwrap();
}