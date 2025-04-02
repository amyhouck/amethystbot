-- Add migration script here
ALTER TABLE `custom_gifs`
  ADD COLUMN `description` VARCHAR(100),
  ADD COLUMN `filename` VARCHAR(30) NOT NULL,
  DROP COLUMN `gif_url`