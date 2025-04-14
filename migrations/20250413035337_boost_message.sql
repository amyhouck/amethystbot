-- Add migration script here
CREATE TABLE `boost` (
  `guild_id` BIGINT UNSIGNED NOT NULL,
  `channel_id` BIGINT UNSIGNED,
  `image_url` VARCHAR(255),
  `message` VARCHAR(500),
  PRIMARY KEY (`guild_id`)
);
INSERT INTO boost (guild_id) SELECT guild_id FROM guild_settings