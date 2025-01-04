-- Add migration script here
-- ALTER TABLE books ADD COLUMN fb2_sha1 VARCHAR(255);
-- WARNING! Run migrate_files.py script before this
ALTER TABLE books DROP COLUMN fb2;
