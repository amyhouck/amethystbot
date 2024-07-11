// Logging type container
pub enum LogType {
    COMMAND_EXECUTION { guild_id: Option<u64>, channel_id: u64, user_id: u64, command_name: String },
    COMMAND_ERROR { guild_id: Option<u64>, channel_id: u64, user_id: u64, command_name: String },
}

// Logging function
pub fn log(log_info: LogType) {
    match log_info {
        LogType::COMMAND_EXECUTION { guild_id, channel_id, user_id, command_name } => {
            let command_location = if guild_id.is_none() {
                String::from("User DM")
            } else {
                format!("Guild ID: {}", guild_id.unwrap())
            };

            let msg = format!("[ LOG ] Command execution - {command_location} - Channel ID: {channel_id} - User ID: {user_id} - Command: {command_name}");
            println!("{msg}");
        },
        LogType::COMMAND_ERROR { guild_id, channel_id, user_id, command_name } => {
            
        }
    }
}