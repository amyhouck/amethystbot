-- Add migration script here
ALTER TABLE `quotes`
    ADD COLUMN sayer_display_name VARCHAR(32) NOT NULL DEFAULT "",
    ADD COLUMN adder_display_name VARCHAR(32) NOT NULL DEFAULT ""