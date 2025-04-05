-- Add migration script here
ALTER TABLE `custom_gifs`
  DROP COLUMN `filename`,
  DROP COLUMN `description`,
  ADD COLUMN `gif_url` TEXT NOT NULL DEFAULT "",
  ADD COLUMN `gif_name` VARCHAR(30) NOT NULL