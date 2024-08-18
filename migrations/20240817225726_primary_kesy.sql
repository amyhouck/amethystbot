-- Add migration script here
ALTER TABLE users ADD PRIMARY KEY (guild_id, user_id);
ALTER TABLE birthday ADD PRIMARY KEY (guild_id, user_id);
ALTER TABLE guild_settings ADD PRIMARY KEY (guild_id);
ALTER TABLE welcome ADD PRIMARY KEY (guild_id);