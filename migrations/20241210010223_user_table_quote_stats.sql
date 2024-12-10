-- Add migration script here
ALTER TABLE `users`
    ADD COLUMN `times_quoted` INT UNSIGNED NOT NULL DEFAULT 0,
    ADD COLUMN `quotes_added` INT UNSIGNED NOT NULL DEFAULT 0