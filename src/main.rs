use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use chrono::{Utc, Duration};
use rand::{Rng, thread_rng};

//mod pokemon;
mod birthday;

pub struct Data { // User data, which is stored and accessible in all command invocations
    database: sqlx::MySqlPool,
    birthday_gifs: Vec<String>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn listener(ctx: &serenity::Context, event: &poise::Event<'_>, data: &Data) -> Result<(), Error> {
    match event {
        poise::Event::Ready { .. } => {
            println!("AmethystBot online!");
            birthday_check(ctx, data).await;
        },
        poise::Event::GuildCreate { guild, .. } => {
            let guild_id = *guild.id.as_u64() as i64;
            let count = sqlx::query!("SELECT COUNT(guild_id) AS count FROM guild_settings WHERE guild_id = ?", guild_id)
                .fetch_one(&data.database)
                .await
                .unwrap();

            if count.count == 0 {
                sqlx::query!("INSERT INTO guild_settings (guild_id) VALUES (?)", guild_id)
                    .execute(&data.database)
                    .await
                    .unwrap();

                println!("[GUILD] Joined new guild: {} (ID: {})", guild.name, guild.id.as_u64());
            }
        },
        _ => {}
    }

    Ok(())
}

async fn birthday_check(ctx: &serenity::Context, data: &Data) {
    loop {
        let current_time = Utc::now();

        if current_time.format("%H:%M").to_string() == String::from("10:00") {
            let registered_guild_channels = sqlx::query!("SELECT guild_id, birthday_channel FROM guild_settings")
                .fetch_all(&data.database)
                .await
                .unwrap();

            let current_date = Utc::now().format("%m-%d").to_string();
            let current_date: Vec<u8> = current_date.split('-').map(|i| i.parse::<u8>().unwrap()).collect();
            println!("{:?}", current_date);

            for guild in registered_guild_channels {
                if guild.birthday_channel.is_none() {
                    continue;
                }
                let channel_id = serenity::ChannelId(guild.birthday_channel.unwrap());

                let guild_birthdays = sqlx::query!("SELECT * FROM birthday WHERE guild_id = ? AND birthday = ? AND birthmonth = ?", guild.guild_id, current_date[1], current_date[0])
                    .fetch_all(&data.database)
                    .await
                    .unwrap();

                for birthday in guild_birthdays {
                    let username = birthday.nickname.unwrap_or(serenity::UserId(birthday.user_id).to_user(&ctx.http).await.unwrap().name);
                    let bday_msg = format!("Happy birthday, {username}! :birthday: We hope you have a great day!");

                    let random_gif = {
                        let mut rng = thread_rng();
                        rng.gen_range(0..data.birthday_gifs.len())
                    };

                    channel_id.send_message(&ctx, |m| {
                        m.content("@everyone :birthday:");
                        m.embed(|e| {
                            e.colour(0xFF0095);
                            e.thumbnail("https://media.istockphoto.com/vectors/birthday-cake-vector-isolated-vector-id901911608?k=6&m=901911608&s=612x612&w=0&h=d6v27h_mYUaUe0iSrtoX5fTw-2wGVIY4UTbQPeI-T5k=");
                            e.title(bday_msg);
                            e.image(&data.birthday_gifs[random_gif])
                        })
                    }).await.unwrap();
                }
            }
        }

        let next_time = (current_time + Duration::days(1)).date_naive().and_hms_opt(10,0,0).unwrap();
        let duration = next_time.signed_duration_since(current_time.naive_utc());
        println!("[ BIRTHDAY CHECK ] Duration until next check: {duration} - {next_time}");

        tokio::time::sleep(duration.to_std().unwrap()).await;
    }
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");
    let database = sqlx::mysql::MySqlPool::connect(&database_url).await.unwrap();

    let birthday_gifs: Vec<String> = vec![
        "https://media.giphy.com/media/WRL7YgP42OKns22wRD/giphy.gif".to_string(),
        "https://media.giphy.com/media/g5R9dok94mrIvplmZd/giphy.gif".to_string(),
        "https://media.giphy.com/media/l4KhS0BOFBhU2SYIU/giphy.gif".to_string(),
        "https://media.giphy.com/media/l4KibWpBGWchSqCRy/giphy.gif".to_string(),
        "https://media.giphy.com/media/arGdCUFTYzs2c/giphy.gif".to_string(),
    ];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                //pokemon::poke_commands::starter()
                birthday::bday(),
            ],
            event_handler: |ctx, event, _, data| Box::pin(listener(ctx, event, data)),
            ..Default::default()
        })
        .token(std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN"))
        .intents(serenity::GatewayIntents::non_privileged())
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    database,
                    birthday_gifs,
                })
            })
        });

    framework.run().await.unwrap();
}