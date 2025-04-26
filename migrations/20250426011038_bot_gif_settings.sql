-- Add migration script here
ALTER TABLE bot_settings
  ADD COLUMN roulette_click_gif TEXT NOT NULL,
  ADD COLUMN roulette_fire_gif TEXT NOT NULL;
UPDATE bot_settings
  SET roulette_click_gif = "http://p6mapclo.s2.myws.ca/panik_gifs/roulette_click.gif";
UPDATE bot_settings
  SET roulette_fire_gif = "http://p6mapclo.s2.myws.ca/panik_gifs/roulette_fire.gif";
