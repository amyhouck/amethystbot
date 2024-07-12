use crate::Context;

// Logging type container
pub enum LogType<'a> {
    CommandExecution { ctx: Context<'a> },
    CommandError { ctx: Context<'a>, error_msg: String },
}

// Logging function
pub fn write_log(log_info: LogType) {
    match log_info {
        LogType::CommandExecution { ctx } => {
            let command_location = if ctx.guild_id().is_none() {
                String::from("User DM")
            } else {
                format!("Guild ID: {}", ctx.guild_id().unwrap().get())
            };

            let msg = format!("[ LOG ] Command execution - {command_location} - Channel ID: {} - User ID: {} - Command: {}",
                ctx.channel_id().get(),
                ctx.author().id.get(),
                ctx.command().name,
            );
            println!("{msg}");
        },
        LogType::CommandError { ctx, error_msg } => {
            let command_location = if ctx.guild_id().is_none() {
                String::from("User DM")
            } else {
                format!("Guild ID: {}", ctx.guild_id().unwrap().get())
            };

            let msg = format!("[ LOG ] Command error - {command_location} - Channel ID: {} - User ID: {} - Command: {}",
                ctx.channel_id().get(),
                ctx.author().id.get(),
                ctx.command().name,
            );
            println!("{msg}");

            let msg = format!("[ LOG ] Error: {error_msg}");
            println!("{msg}");
        }
    }
}