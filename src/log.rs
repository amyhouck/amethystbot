use crate::Context;

// Logging type container
pub enum LogType<'a> {
    // Command Types
    CommandExecution { ctx: Context<'a> },
    CommandError { ctx: Context<'a>, error_msg: String },

    // Birthday Module
    BirthdayTimerReset { duration: String },

    // Welcome Message Module
    WelcomeNewUser { guild_id: u64 },

    // Bot Logging
    BotGuildLogin { guild_id: u64 },
    // BotGuildJoin { guild_id: u64 },
    BotGuildDBRegister { guild_id: u64, table_name: String },
    BotStartup,

    // User Logging
    UserDBRegister { guild_id: u64, user_id: u64 },
    UserDBRemove,

    // Database Logging
    DBError { db_error: String },

    // MTG Module Logging
    MTGScryfallError { error: String },
    MTGScryfallParsingError { error: String },
}

// Logging function
pub fn write_log(log_info: LogType) {
    match log_info {
        // Command Types
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

            let msg = format!("[ LOG ] Command error - {command_location} - Channel ID: {} - User ID: {} - Command String: {}",
                ctx.channel_id().get(),
                ctx.author().id.get(),
                ctx.invocation_string(),
            );
            println!("{msg}");

            let msg = format!("[ LOG ] Error: {error_msg}");
            println!("{msg}");
        },

        // Birthday Module
        LogType::BirthdayTimerReset { duration } => {
            let msg = format!("[ BIRTHDAY ] Duration until next birthday check: {duration}");
            println!("{msg}");
        },

        // Welcome Message Module
        LogType::WelcomeNewUser { guild_id } => {
            let msg = format!("[ WELCOME ] New welcome message posted - Guild ID: {guild_id}");
            println!("{msg}");
        },

        // Bot Events
        LogType::BotGuildLogin { guild_id } => {
            let msg = format!("[ BOT ] Logged into guild - Guild ID: {guild_id}");
            println!("{msg}");
        },

        // LogType::BotGuildJoin { guild_id } => {
        //     let msg = format!("[ BOT ] Joined new guild - Guild ID: {guild_id}");
        //     println!("{msg}");
        // },

        LogType::BotGuildDBRegister { guild_id, table_name} => {
            let msg = format!("[ BOT ] Registering new guild into table \"{table_name}\" - Guild ID: {guild_id}");
            println!("{msg}");
        },
        
        LogType::BotStartup => {
            let msg = format!("[ BOT ] AmethystBot is online!");
            println!("{msg}");
        },

        // User Logging
        LogType::UserDBRegister { guild_id, user_id } => {
            let msg = format!("[ USER ] New user added to database - Guild ID: {guild_id} - User ID: {user_id}");
            println!("{msg}");
        },

        LogType::UserDBRemove => {
            let msg = format!("[ USER ] User left a server and associated data has been removced.");
            println!("{msg}");
        },

        // Database Logging
        LogType::DBError { db_error } => {
            let msg = format!("[ DATABASE - ERROR ] An error occurred trying to query the databse: {db_error}");
            println!("{msg}");
        },

        // MTG Module
        LogType::MTGScryfallError { error } => {
            let msg = format!("[ MTG ] An error occurred with data from Scryfall: {error}");
            println!("{msg}");
        },

        LogType::MTGScryfallParsingError { error } => {
            let msg = format!("[ MTG ] An error occurred trying to parse the Scryfall data: {error}");
            println!("{msg}");
        }
    }
}