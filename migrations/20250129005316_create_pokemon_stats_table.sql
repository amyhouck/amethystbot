-- Add migration script here
CREATE TABLE `pokemon_base_info` (
    `id` INT UNSIGNED NOT NULL,
    `name` VARCHAR(64) NOT NULL,
    `form` VARCHAR(128) NOT NULL,
    `type_1` INT NOT NULL,
    `type_2` INT NOT NULL,
    `base_hp` INT UNSIGNED NOT NULL,
    `base_attack` INT UNSIGNED NOT NULL,
    `base_defense` INT UNSIGNED NOT NULL,
    `base_special_attack` INT UNSIGNED NOT NULL,
    `base_special_defense` INT UNSIGNED NOT NULL,
    `base_speed` INT UNSIGNED NOT NULL,
    PRIMARY KEY (`id`, `name`, `form`)
)