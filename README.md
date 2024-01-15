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