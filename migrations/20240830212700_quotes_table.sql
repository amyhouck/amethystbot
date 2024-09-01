-- Add migration script here
CREATE TABLE `quotes` (
    `guild_id` BIGINT UNSIGNED NOT NULL,
    `adder_id` BIGINT UNSIGNED NOT NULL,
    `sayer_id` BIGINT UNSIGNED NOT NULL,
    `quote_id` INT UNSIGNED NOT NULL,
    `quote` VARCHAR(500) NOT NULL,
    `timestamp` DATE NOT NULL,
    PRIMARY KEY (`guild_id`, `quote_id`)
)