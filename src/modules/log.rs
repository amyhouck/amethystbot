use crate::Context;

// Logging type container
pub enum LogType<'a> {
    // Command Types
    CommandExecution { ctx: Context<'a> },
    // CommandError { ctx: Context<'a> },

    // Birthday Module
    BirthdayTimerReset { duration: String },

    // Welcome Message Module
    WelcomeNewUser { guild_id: u64 },

    // Bot Logging
    BotGuildLogin { guild_id: u64 },
    BotGuildDBRegister { guild_id: u64, table_name: String },
    BotStartup,
    BotShutdown,

    // User Logging
    UserDBRegister { guild_id: u64, user_id: u64 },
    UserDBNameChange { guild_id: u64, user_id: u64 },
    UserDBRemove,

    // Database Logging
    DBError { db_error: String },

    // MTG Module Logging
    MTGScryfallError { error: String },
    MTGScryfallParsingError { error: String },

    // VCTracker Module
    VCTrackerSafeguardAdjustment { guild_id: u64, user_id: u64 },
    VCTrackerSafeguardSkip { guild_id: u64, user_id: u64},
    VCTrackerResetMonthlyComplete,
    VCTrackerResetMonthlyDuration { duration: String },
}

// Logging function
pub fn write_log(log_info: LogType) {
    let msg = match log_info {
        // Command Types
        LogType::CommandExecution { ctx } => {
            let command_location = if ctx.guild_id().is_none() {
                String::from("User DM")
            } else {
                format!("Guild ID: {}", ctx.guild_id().unwrap().get())
            };

            format!("[ LOG ] Command execution - {command_location} - Channel ID: {} - User ID: {} - Command: {}",
                ctx.channel_id().get(),
                ctx.author().id.get(),
                ctx.command().name,
            )
        },

        // LogType::CommandError { ctx} => {
        //     let command_location = if ctx.guild_id().is_none() {
        //         String::from("User DM")
        //     } else {
        //         format!("Guild ID: {}", ctx.guild_id().unwrap().get())
        //     };

        //     format!("[ LOG ] Command error - {command_location} - Channel ID: {} - User ID: {} - Command String: {}",
        //         ctx.channel_id().get(),
        //         ctx.author().id.get(),
        //         ctx.invocation_string(),
        //     )
        // },

        // Birthday Module
        LogType::BirthdayTimerReset { duration } => format!("[ BIRTHDAY ] Duration until next birthday check: {duration}"),

        // Welcome Message Module
        LogType::WelcomeNewUser { guild_id } => format!("[ WELCOME ] New welcome message posted - Guild ID: {guild_id}"),

        // Bot Events
        LogType::BotGuildLogin { guild_id } => format!("[ BOT ] Logged into guild - Guild ID: {guild_id}"),

        LogType::BotGuildDBRegister { guild_id, table_name} => format!("[ BOT ] Registering new guild into table \"{table_name}\" - Guild ID: {guild_id}"),
        
        LogType::BotStartup => String::from("[ BOT ] AmethystBot is online!"),

        LogType::BotShutdown => String::from("[ BOT ] AmethystBot is shutting down!"),

        // User Logging
        LogType::UserDBRegister { guild_id, user_id } => format!("[ USER ] New user added to database - Guild ID: {guild_id} - User ID: {user_id}"),
        LogType::UserDBNameChange { guild_id, user_id } => format!("[ USER ] Altered saved display name in database - Guild ID: {guild_id} - User ID: {user_id}"),
        LogType::UserDBRemove => String::from("[ USER ] User left a server and associated data has been removed."),

        // Database Logging
        LogType::DBError { db_error } => format!("[ DATABASE - ERROR ] An error occurred trying to query the databse: {db_error}"),

        // MTG Module
        LogType::MTGScryfallError { error } => format!("[ MTG ] An error occurred with data from Scryfall: {error}"),

        LogType::MTGScryfallParsingError { error } => format!("[ MTG ] An error occurred trying to parse the Scryfall data: {error}"),

        // VCTracker Module
        LogType::VCTrackerSafeguardAdjustment { guild_id, user_id } => format!("[ VCTRACKER ] SAFEGUARD - Adjusted time for User ID ({user_id}) - Guild ID: {guild_id}"),

        LogType::VCTrackerSafeguardSkip { guild_id, user_id } => format!("[ VCTRACKER ] SAFEGUARD - Skipping user's time update. Guild ID: {guild_id} - User ID: {user_id}"),

        LogType::VCTrackerResetMonthlyComplete => String::from("[ VCTRACKER ] Reset monthly VC times for every user."),

        LogType::VCTrackerResetMonthlyDuration { duration } => format!("[ VCTRACKER ] Duration until next monthly leaderboard reset: {duration}"),
    };

    println!("{msg}");
}