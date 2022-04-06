-- Add migration script here

ALTER TABLE books
ADD COLUMN series unsigned bigint;
ALTER TABLE books
ADD COLUMN series_title VARCHAR(255);
