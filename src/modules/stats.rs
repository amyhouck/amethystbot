use crate::{data, Context, Error};
use crate::data::{user_table_check, User};
use poise::serenity_prelude as serenity;

//---------------------
// Stat embed page builders
//---------------------
fn build_general_embed(
    amethyst_user: &User,
    user_avatar: &String,
    vctime: String,
    quotes_added: i64,
    times_quoted: i64,
) -> serenity::CreateEmbed {
    let embed_description = format!("
        **Time spent in VC:** {vctime}
        
        **Quotes added:** {quotes_added}
        **Times quoted:** {times_quoted}"
    );
        
    serenity::CreateEmbed::default()
        .title(format!("{}'s Stats", &amethyst_user.display_name))
        .thumbnail(user_avatar)
        .description(embed_description)
        .colour(0x8CAAC2)
}

fn build_misc_embed(
    user_data: &User,
    user_avatar: &String,
) -> serenity::CreateEmbed {
    let embed_description = format!("
        **Cookies sent:** {cookie_sent}
        **Cookies received:** {cookie_received}
        
        **Cakes sent:** {cake_sent}
        **Cakes received:** {cake_received}
        **Times GLaDOSed:** {cake_glados}
        
        **Cups of tea given:** {tea_sent}
        **Cups of tea received:** {tea_received}
        
        **People slapped:** {slap_sent}
        **Slaps received:** {slap_received}",
        
        cookie_sent = user_data.cookie_sent,
        cookie_received = user_data.cookie_received,
        cake_sent = user_data.cake_sent,
        cake_received = user_data.cake_received,
        cake_glados = user_data.cake_glados,
        slap_sent = user_data.slap_sent,
        slap_received = user_data.slap_received,
        tea_sent = user_data.tea_sent,
        tea_received = user_data.tea_received
    );
    
    serenity::CreateEmbed::default()
        .title(format!("{}'s Stats (Misc.)", &user_data.display_name))
        .thumbnail(user_avatar)
        .description(embed_description)
        .colour(0x8CAAC2)
}

//---------------------
// Commands
//---------------------
/// Check user stats
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
)]
pub async fn stats(
    ctx: Context<'_>,
    #[description = "The user you want stats for."] user: Option<serenity::User>
) -> Result<(), Error> {
    ctx.defer().await?;

    // Get user id and check database
    let user = match user {
        Some(u) => u,
        None => ctx.author().clone()
    };
    
    let user_id = user.id;
    let guild_id = ctx.guild_id().unwrap().get();
    user_table_check(ctx, &user).await;

    // Update Voice Time
    let vc_info = ctx.guild().unwrap().voice_states.clone();
    let vc_info = vc_info.get(&user_id);

    if vc_info.is_some() {
        match crate::vctracker::recheck_time(vc_info.unwrap(), &ctx.data().database).await {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
    }

    // Grab quote counts
    let quote_data = sqlx::query!("SELECT CAST(SUM(IF(adder_id = ?, 1, 0)) AS INTEGER) AS quotes_added, CAST(SUM(IF(sayer_id = ?, 1, 0)) AS INTEGER) AS times_quoted FROM quotes WHERE guild_id = ?", user_id.get(), user_id.get(), guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();
    
    // Build stats embeds
    let user_data = sqlx::query_as!(User, "SELECT * FROM users WHERE guild_id = ? AND user_id = ?", guild_id, user_id.get())
        .fetch_one(&ctx.data().database)
        .await
        .unwrap();

    let vctime = format!("{}h {}m {}s",
        (user_data.vctrack_total_time / 60) / 60,
        (user_data.vctrack_total_time / 60) % 60,
        user_data.vctrack_total_time % 60,
    );
    
    let avatar_url = user.avatar_url().unwrap_or(String::new());
    let stat_embeds: [serenity::CreateEmbed; 2] = [
        build_general_embed(&user_data, &avatar_url, vctime, quote_data.quotes_added.unwrap(), quote_data.times_quoted.unwrap()),
        build_misc_embed(&user_data, &avatar_url)
    ];
    
    // Build interaction
    let mut stats_page = 0;
    let ctx_id = ctx.id();
    let gen_id = format!("{ctx_id}gen");
    let misc_id = format!("{ctx_id}misc");
    
    let buttons: Vec<serenity::CreateButton> = vec![
        serenity::CreateButton::new(&gen_id).label("General"),
        serenity::CreateButton::new(&misc_id).label("Miscellaneous")  
    ];
    let buttons = serenity::CreateActionRow::Buttons(buttons);
    
    ctx.send(poise::CreateReply::default()
        .embed(stat_embeds[stats_page].clone())
        .components(vec![buttons])
    ).await?;
    
    // Handle interaction
    while let Some(press) = serenity::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(600))
        .await
    {
        if press.data.custom_id == gen_id {
            stats_page = 0;
        } else if press.data.custom_id == misc_id {
            stats_page = 1;
        } else {
            continue;
        }
        
        press.create_response(
            ctx.serenity_context(),
            serenity::CreateInteractionResponse::UpdateMessage(
                serenity::CreateInteractionResponseMessage::new()
                    .embed(stat_embeds[stats_page].clone())   
            )
        ).await?;
    }
    
    Ok(())
}

/// Get total stats of the server.
#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5
)]
pub async fn serverstats(ctx: Context<'_>) -> Result<(), Error> {
    // `users` table data
    let guild_id = ctx.guild_id().unwrap().get();

    let server_data = sqlx::query!("SELECT * FROM users WHERE guild_id = ?", guild_id)
        .fetch_all(&ctx.data().database)
        .await
        .unwrap();

    // If no data, return msg.
    if server_data.is_empty() {
        ctx.say("This server does not have any stats available yet!").await?;
        return Ok(());
    }

    // `quotes` table data
    let quote_count = sqlx::query!("SELECT COUNT(quote_id) AS count FROM quotes WHERE guild_id = ?", guild_id)
        .fetch_one(&ctx.data().database)
        .await
        .unwrap()
        .count;

    // Server stats struct construction
    let mut server_stats = data::ServerStats::default();
    let mut raw_vc_time = 0u32;
    for record in server_data {
        server_stats.bomb_defused += record.bomb_defused;
        server_stats.bomb_failed += record.bomb_failed;
        server_stats.bomb_sent += record.bomb_sent;
        server_stats.cake_sent += record.cake_sent;
        server_stats.cookie_sent += record.cookie_sent;
        server_stats.tea_sent += record.tea_sent;
        server_stats.slap_sent += record.slap_sent;
        server_stats.glados_appearances += record.cake_glados;
        raw_vc_time += record.vctrack_total_time;
    }

    let formatted_vc_time = format!("{}d {}h {}m {}s",
        ((raw_vc_time / 60) / 60) / 24,
        ((raw_vc_time / 60) / 60) % 24,
        (raw_vc_time / 60) % 60,
        raw_vc_time % 60
    );

    // Build and send stats embed
    let embed_desc = format!("**Total VC time:** {formatted_vc_time}\n\n**Cookies sent:** {0}\n**Cakes sent:** {1}\n**Tea sent:** {2}\n**Slaps sent:** {3}\n**GLaDOS appearances:** {7}\n**Total quotes:** {quote_count}\n\n**Bombs sent:** {4}\n**Bombs defused:** {5}\n**Bombs exploded:** {6}",
        server_stats.cookie_sent,
        server_stats.cake_sent,
        server_stats.tea_sent,
        server_stats.slap_sent,
        server_stats.bomb_sent,
        server_stats.bomb_defused,
        server_stats.bomb_failed,
        server_stats.glados_appearances,
    );

    let mut embed = serenity::CreateEmbed::new()
        .title("Server Stats")
        .colour(0x8caac2)
        .description(embed_desc);

    if ctx.guild().unwrap().icon_url().is_some() {
        embed = embed.thumbnail(ctx.guild().unwrap().icon_url().unwrap());
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}