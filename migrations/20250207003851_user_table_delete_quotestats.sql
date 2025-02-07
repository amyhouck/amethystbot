-- Add migration script here
ALTER TABLE `users`
    DROP COLUMN `times_quoted`,
    DROP COLUMN `quotes_added`