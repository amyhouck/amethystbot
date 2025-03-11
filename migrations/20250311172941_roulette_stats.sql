-- Add migration script here
ALTER TABLE `users`
  ADD COLUMN `roulette_deaths` INT UNSIGNED NOT NULL DEFAULT 0