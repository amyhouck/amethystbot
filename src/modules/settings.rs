use crate::{Context, Error};

#[derive(poise::ChoiceParameter)]
enum EnableDisable {
    Enable = 1,
    Disable = 0
}

impl std::fmt::Display for EnableDisable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EnableDisable::Enable => write!(f, "enable"),
            EnableDisable::Disable => write!(f, "disable")
        }
    }
}

#[poise::command(
    slash_command,
    guild_only,
    member_cooldown = 5,
    subcommands("command_ping")
)]
pub async fn settings(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Enable or disable pings on some commands
#[poise::command(
    slash_command,
    ephemeral = true
)]
pub async fn command_ping(
    ctx: Context<'_>,
    
    #[description = "Enable or disable pings"]
    choice: EnableDisable
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get();
    let user_id = ctx.author().id.get();
    let bool_choice: bool = match &choice {
        &EnableDisable::Enable => true,
        &EnableDisable::Disable => false  
    };
    
    sqlx::query!("UPDATE user_settings SET command_ping = ? WHERE guild_id = ? AND user_id = ?", bool_choice, guild_id, user_id)
        .execute(&ctx.data().database)
        .await
        .unwrap();
        
    let msg = match choice {
          EnableDisable::Enable => "You have **enabled** pings for certain commands!",
          EnableDisable::Disable => "You have **disabled** pings for certain commands!"
    };
    
    ctx.say(msg).await?;
    
    Ok(())
}