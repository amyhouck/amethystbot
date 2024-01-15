-- Add migration script here
CREATE TABLE guild_settings (
    guild_id BIGINT UNSIGNED NOT NULL,
    birthday_channel BIGINT UNSIGNED
)