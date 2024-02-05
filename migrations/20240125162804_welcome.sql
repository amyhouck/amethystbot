-- Add migration script here
CREATE TABLE welcome (
    guild_id BIGINT UNSIGNED NOT NULL,
    channel_id BIGINT UNSIGNED,
    image_url TEXT,
    message TEXT
)