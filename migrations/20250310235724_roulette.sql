-- Add migration script here
ALTER TABLE `guild_settings`
  ADD COLUMN `roulette_chamber` TINYINT UNSIGNED NOT NULL DEFAULT 0,
  ADD COLUMN `roulette_count` TINYINT UNSIGNED NOT NULL DEFAULT 0