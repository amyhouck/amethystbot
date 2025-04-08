-- Add migration script here
CREATE TABLE `user_settings` (
  `guild_id` BIGINT UNSIGNED NOT NULL,
  `user_id` BIGINT UNSIGNED NOT NULL,
  `command_ping` BOOL NOT NULL DEFAULT true,
  PRIMARY KEY(`guild_id`, `user_id`)
);
INSERT INTO user_settings (guild_id, user_id) SELECT guild_id, user_id FROM users