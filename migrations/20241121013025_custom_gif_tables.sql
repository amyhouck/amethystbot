-- Add migration script here
CREATE TABLE `custom_gifs` (
    `guild_id` BIGINT UNSIGNED NOT NULL,
    `gif_type` VARCHAR(16) NOT NULL,
    `gif_id` INT UNSIGNED NOT NULL,
    `gif_url` TEXT NOT NULL,
    PRIMARY KEY(`guild_id`, `gif_type`, `gif_id`)
)