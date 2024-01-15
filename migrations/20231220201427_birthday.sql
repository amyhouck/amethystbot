-- Add migration script here
CREATE TABLE birthday (
    guild_id BIGINT UNSIGNED NOT NULL,
    user_id BIGINT UNSIGNED NOT NULL,
    birthday TINYINT UNSIGNED NOT NULL,
    birthmonth TINYINT UNSIGNED NOT NULL
)