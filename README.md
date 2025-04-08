# AmethystBot

My personal Discord bot built for friends and to do miscellaneous things.

## Commands
() denotes required parameters whereas [] denotes optional parameters

### Birthday Module
The set of commands that handle user birthdays. You **MUST** set a channel with `/bday setchannel` in order to run these commands.

- `/bday setchannel (channel)` - Sets a channel for the bot to post all birthday updates. (Requires MANAGE_CHANNELS permission)
- `/bday add (user) (birthmonth 1-12) (birthday 1-31) [name]` - Adds a user's birtday. Set [name] to use a customized name. Max characters for the custom name is 30. (Requires MANAGE_CHANNELS permission)
- `/bday edit (user) [birthmonth 1-12] [birthday 1-31] [name]` - Modifies a user's birthday. (Requires MANAGE_CHANNELS permission)
- `/bday remove [user] [userID]` - Removes a user's birthday using either their username or their userID. (Requires MANAGE_CHANNELS permission)
- `/bday info (user)` - Returns the user's birthday and their custom name if set.

### CustomGIFs Module
This module allows users to specify custom GIFs for various commands. By default, no GIFs are saved for the options available here. If some are set, the commands
that use these GIFs will select one at random.

- `/addgif (GIF TYPE) (NAME) (GIF URL)` - Adds a GIF URL to the database.
- `/delgif (GIF TYPE) (ID)` - Removes a specific GIF from the database.
- `/listgifs (GIF TYPE)` - Displays all saved GIFs of a certain type.
- `/setgifrole [ROLE]` - Sets or unsets the role required to use this module. (Requires MANAGE_CHANNELS permission)

### Miscellaneous Module
Random commands for funsies.

- `/cookie (USER)` - Gives a cookie to someone.
- `/tea (USER)` - Gives someone some tea.
- `/cake (USER)` - Probably gives a cake to a user.
- `/slap (USER)` - Slap someone like in the good ol' days of IRC.

### MTG Module
A set of commands for Magic: The Gathering.

- `/mtg card [NAME] [SET CODE] [COLLECTOR NUMBER]` - Displays information and legalities about a card from Scryfall. All parameters are set to be optional,
    but if the set code and collector number are not set, the name is required. If specifying the set code, the name or collector number is required.
    A set code is required if only specifying a collector number.

### Quotes Module
The quotes module allows you to save memorable quotes by users on the server without worrying about pin limitations.

- `/addquote (SAYER) (QUOTE) [DATE]` -  Adds a quote to the database. Adding a date allows you to backdate the quote if was quoted a while ago.
- `/delquote (ID)` - Deletes a quote from the database based on ID. Quote IDs are not static. They will be adjusted as quotes are deleted.
- `/quote [ID] [USER] [TEXT]` - Pulls a quote from the database. You can only use 1 option at a time. Leaving them blank will pull a random quote, only specifying
    a user will pull a random quote by that user, the text field allows you to pull a quote by text, and the ID will pull a specific quote by ID.
- `/listquotes [USER]` - Lists all quotes saved for a server. If user is specified, list all quotes made by the user on the server.
- `/setquoterole [ROLE]` - Optionally requires a user to have a role in order to use the quotes module. (Requires MANAGE_CHANNELS permission)

### Settings Module
Contains various settings for the bot.
- `/settings command_ping (ENABLE/DISABLE)` - Enables or disables being pinged for various commands. Currently supports `/slap`, `/cookie`, `/tea`, `/cake`.

### Stats Module
The stats module just displays stats for a specific user or the whole server.

- `/stats [USER]` - Grabs the individual stats for a user. Leaving blank will grab your own stats.
- `/serverstats` - Displays the combined stats for the whole server.

### VCTracker Module
This module keeps track of the amount of time users spend in VC and contains a leaderboard. You can specify a channel to ignore that will not track time for a user.

- `/vctracker ignorechannel [CHANNEL]` - Sets or unsets a channel to ignore for tracking time spent.
- `/vctop` - Displays the top 10 users in a server for most amount of time spent in VC for all-time or monthly.

### Welcome Module
This module handles custom welcome messages for a server. Setting a channel for welcome messages to be posted to is **required** before using these.

- `/welcome setmessage [MESSAGE]` - Sets or unsets the message that shows when a user joins the server.
- `/welcome setimage [IMAGE_URL]` - Sets or unsets the image displayed inside the welcome embed.
- `/welcome setchannel [CHANNEL]` - Sets or unsets the channel where welcome messages will be posted. Leaving this blank will disable messages. (Requires MANAGE_CHANNELS permission)
- `/setleavechannel [CHANNEL]` - Sets the channel to send a message when a user leaves a server. Leaving this blank will disable messages. (Requires MANAGE_CHANNEL permission)

## Minigames
Fun little minigames in the bot.

### Bomb
A user will be sent a bomb embed with options to pick a colored wire. They have 20 seconds to disarm it and only 2 chances.

- `/bomb (USER)` - Sends the bomb to someone.

### Rock, Paper, Scissors
Play Rock, Paper, Scissors with someone.

- `/rps (USER)` - Starts a game of Rock, Paper, Scissors with someone.

### Roulette
This is Russian Roulette. A number is set for a server from 1 to 6 and the number of tries will increment every time the command is used until it matches
the set number. The amount of tries will not be displayed.

- `/roulette` - Pulls the trigger of the gun in Russian roulette.