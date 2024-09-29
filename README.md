Not the prettiest code I've written, but it works.

This Bot is made for [LIBAFO](https://www.twitch.tv/libafo) and a special event he sometimes streams.

The name 'ddf' comes from the german "Der dümmste fliegt", which basically translates to "dumbest one out".

The Bot resembles a 'Game Manager'. Only one game is able to run at a time, so I opted for a simple Mutex to modify the state (of the game and such) used across the bot.