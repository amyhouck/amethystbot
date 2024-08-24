-- Add migration script here
CREATE TABLE `birthday` (
  `guild_id` bigint(20) UNSIGNED NOT NULL,
  `user_id` bigint(20) UNSIGNED NOT NULL,
  `birthday` tinyint(3) UNSIGNED NOT NULL,
  `birthmonth` tinyint(3) UNSIGNED NOT NULL,
  `nickname` varchar(30) DEFAULT NULL,
  PRIMARY KEY (`guild_id`, `user_id`)
);

CREATE TABLE `guild_settings` (
  `guild_id` bigint(20) UNSIGNED NOT NULL,
  `birthday_channel` bigint(20) UNSIGNED DEFAULT NULL,
  `birthday_role` bigint(20) UNSIGNED DEFAULT NULL,
  `vctrack_ignored_channel` bigint(20) UNSIGNED DEFAULT NULL,
  PRIMARY KEY (`guild_id`)
);

CREATE TABLE `users` (
  `guild_id` bigint(20) UNSIGNED NOT NULL,
  `user_id` bigint(20) UNSIGNED NOT NULL,
  `cookie_sent` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `cookie_received` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `slap_sent` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `slap_received` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `cake_sent` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `cake_received` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `cake_glados` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `tea_sent` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `tea_received` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `bomb_sent` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `bomb_defused` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `bomb_failed` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `vctrack_join_time` int(10) UNSIGNED NOT NULL DEFAULT 0,
  `vctrack_total_time` int(10) UNSIGNED NOT NULL DEFAULT 0,
  PRIMARY KEY (`guild_id`, `user_id`)
);

CREATE TABLE `welcome` (
  `guild_id` bigint(20) UNSIGNED NOT NULL,
  `channel_id` bigint(20) UNSIGNED DEFAULT NULL,
  `image_url` text DEFAULT NULL,
  `message` text DEFAULT NULL,
  PRIMARY KEY (`guild_id`)
)