-- Add migration script here
CREATE TABLE `pokemon_move_info` (
    `id` INT UNSIGNED NOT NULL AUTO_INCREMENT,
    `name` VARCHAR(128) NOT NULL,
    `type` INT NOT NULL,
    `category` INT UNSIGNED NOT NULL,
    `power` INT UNSIGNED NOT NULL,
    `accuracy` INT UNSIGNED NOT NULL,
    `pp` INT UNSIGNED NOT NULL,
    `priority` INT UNSIGNED NOT NULL,

    PRIMARY KEY (`id`)
)