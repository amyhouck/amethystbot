-- Add migration script here
ALTER TABLE `users` ADD COLUMN display_name VARCHAR(32) NOT NULL DEFAULT ""