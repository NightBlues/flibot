-- Add migration script here
DROP TABLE IF EXISTS authors;
CREATE TABLE IF NOT EXISTS authors (
  id unsigned bigint NOT NULL UNIQUE,
  name VARCHAR(255) NOT NULL,
  url VARCHAR(255) NOT NULL,
  books_list_fetched boolean NOT NULL,
  last_update datetime NOT NULL
);

DROP TABLE IF EXISTS books;
CREATE TABLE IF NOT EXISTS books (
  id unsigned bigint NOT NULL UNIQUE,
  author unsigned bigint NOT NULL REFERENCES authors(id) ON DELETE RESTRICT,
  title VARCHAR(255) NOT NULL,
  mark float,
  annotation VARCHAR(4096),
  cover_url VARCHAR(255),
  cover BLOB,
  fb2_url VARCHAR(255) NOT NULL,
  fb2_filename VARCHAR(255),
  fb2 BLOB
);
