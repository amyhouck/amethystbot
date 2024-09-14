-- Add migration script here
ALTER TABLE `users` ADD COLUMN `vctrack_monthly_time` INT UNSIGNED NOT NULL DEFAULT 0