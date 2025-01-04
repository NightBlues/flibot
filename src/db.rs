// use std::sync::{
//   Arc,
//   atomic::AtomicPtr,
// };
use anyhow::{
  Error,
  Result,
  Context,
};
use derive_more::{Display};
use sqlx::sqlite::SqlitePool;


#[derive(Clone)]
pub struct Author {
  pub id: i64,
  pub name: String,
  pub url: String,
  pub books_list_fetched: bool,
  // unsupported type DATETIME of column #5 ("last_update")
  // pub last_update: i64,
}

#[derive(Clone)]
pub struct Book {
  pub id: i64,
  pub title: String,
  pub author: i64,
  pub fb2_url: String,
  // optional fields, filled on demand
  pub mark: Option<f64>,
  pub annotation: Option<String>,
  pub cover_url: Option<String>,
  pub cover: Option<Vec<u8>>,
  pub fb2_filename: Option<String>,
  pub fb2_sha1: Option<String>,
  pub series: Option<i64>,
  pub series_title: Option<String>,
}

#[derive(Display, Clone)]
#[display(fmt = "total books: {:?},\nfb2: {:?}\n", total_books, total_fb2)]
pub struct Stats {
  pub total_books: Option<i64>,
  pub total_fb2: Option<i64>,
}


pub async fn get_book(pool: &SqlitePool, id: i64) -> Result<Book> {
  let row = sqlx::query_as!(Book, r#"
  SELECT
    id,
    title,
    author,
    fb2_url,
    mark,
    annotation,
    cover_url,
    cover,
    fb2_filename,
    fb2_sha1,
    series,
    series_title
  FROM books
  WHERE id = $1
  "#, id).fetch_optional(pool)
    .await.context("db::get_book")?;
  let res = row.ok_or_else(|| Error::msg("Nothing found by primary key"))?;

  Ok(res)
}

pub async fn get_books_for_author(pool: &SqlitePool, id: i64
) -> Result<Vec<Book>> {
  let res = sqlx::query_as!(Book, r#"
  SELECT
    id,
    title,
    author,
    fb2_url,
    mark,
    annotation,
    cover_url,
    cover,
    fb2_filename,
    fb2_sha1,
    series,
    series_title
  FROM books
  WHERE author = $1
  "#, id).fetch_all(pool)
    .await.context("db::get_books_for_author")?;
  // let res = row.ok_or_else(|| Error::msg("Nothing found by author"))?;

  Ok(res)
}


pub async fn add_book(
  pool: &SqlitePool, book: Book
) -> Result<Book> {
  let Book {id, title, author, annotation,
            cover_url, fb2_url, mark, series, series_title, ..} = book;
  // let mut conn = pool.acquire().await?;
  let row = sqlx::query_as!(Book, r#"
    INSERT INTO books
    (id, title, author, annotation, cover_url, fb2_url, mark, series, series_title)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
    ON CONFLICT DO UPDATE SET
     annotation=(CASE WHEN annotation IS NULL THEN $4
                 ELSE annotation END),
     cover_url=(CASE WHEN cover_url IS NULL THEN $5
                ELSE cover_url END),
     mark=(CASE WHEN mark IS NULL THEN $7
           ELSE mark END),
     series=(CASE WHEN series IS NULL THEN $8
             ELSE series END),
     series_title=(CASE WHEN series_title IS NULL THEN $9
                   ELSE series_title END)
    RETURNING
       id,
       title,
       author,
       mark,
       annotation,
       cover_url,
       cover,
       fb2_url,
       fb2_filename,
       fb2_sha1,
       series,
       series_title
  "#, id, title, author, annotation, cover_url, fb2_url, mark, series, series_title)
  .fetch_optional(pool)
   // .execute(&mut conn)
    .await.context("db::add_book")?;
  let row = row.ok_or_else(|| Error::msg("Nothing saved"))?;
  // let _id = row.last_insert_rowid();

  Ok(row)
}

pub async fn save_book_cover(
  pool: &SqlitePool, id: i64, cover: bytes::Bytes) -> Result<Book> {
  // let cover : Vec<u8> = cover.as_ptr().into();
  let cover : Vec<u8> = (*cover).into();
  let row = sqlx::query_as!(Book, r#"
    UPDATE books
    SET cover=$1
    WHERE id=$2
    RETURNING
      id,
      title,
      author,
      mark,
      annotation,
      cover_url,
      cover,
      fb2_url,
      fb2_filename,
      fb2_sha1,
      series,
      series_title
  "#, cover, id).fetch_optional(pool)
   .await.context("db::save_cover")?;
  let res = row.ok_or_else(|| Error::msg("Nothing saved"))?;

  Ok(res)
}

pub async fn save_book_fb2(
  pool: &SqlitePool,
  id: i64,
  fb2_filename: String,
  fb2_sha1: String,
) -> Result<Book> {
  // let cover : Vec<u8> = cover.as_ptr().into();
  // let fb2 : Vec<u8> = (*fb2).into();
  let row = sqlx::query_as!(Book, r#"
    UPDATE books
    SET fb2_sha1=$1, fb2_filename=$2
    WHERE id=$3
    RETURNING
      id,
      title,
      author,
      mark,
      annotation,
      cover_url,
      cover,
      fb2_url,
      fb2_filename,
      fb2_sha1,
      series,
      series_title
  "#, fb2_sha1, fb2_filename, id).fetch_optional(pool)
   .await.context("db::save_fb2")?;
  let res = row.ok_or_else(|| Error::msg("Nothing saved"))?;

  Ok(res)
}


pub async fn get_author(
  pool: &SqlitePool, id: i64
) -> Result<Author> {
  let row = sqlx::query_as!(Author, r#"
  SELECT
    id,
    name,
    url,
    books_list_fetched
  FROM authors
  WHERE id = $1
  "#, id).fetch_optional(pool)
    .await.context("db::get_author")?;
  let res = row.ok_or_else(|| Error::msg("Nothing found by primary key"))?;

  Ok(res)
}

pub async fn add_author(
  pool: &SqlitePool,
  author: Author,
) -> Result<Author> {
  let Author {id, name, url, books_list_fetched} = author;
  // let tags =
  // only set to true if given explicit
  // dont reset to false on per-book requests
  let row = sqlx::query_as!(Author, r#"
    INSERT INTO authors
    (id, name, url, books_list_fetched, last_update)
    VALUES ($1, $2, $3, $4, datetime('now'))
    ON CONFLICT DO UPDATE SET
      books_list_fetched = (CASE WHEN $4 THEN true
                            ELSE books_list_fetched END)
    RETURNING id, name, url, books_list_fetched
  "#, id, name, url, books_list_fetched).fetch_optional(pool)
    .await.context("db::add_author")?;
  let res = row.ok_or_else(|| Error::msg("Nothing saved"))?;

  Ok(res)
}

pub async fn search_author(
  pool: &SqlitePool, text: &String
) -> Result<Vec<Author>> {
  let res = sqlx::query_as!(Author, r#"
  SELECT
    id,
    name,
    url,
    books_list_fetched
  FROM authors
  WHERE name LIKE replace('%' || $1 || '%', ' ', '%')
  "#, text).fetch_all(pool)
    .await.context("db::search_author")?;

  Ok(res)
}


pub async fn search_book(
  pool: &SqlitePool, text: &String
) -> Result<Vec<Book>> {
  let res = sqlx::query_as!(Book, r#"
  SELECT
    id,
    title,
    author,
    fb2_url,
    mark,
    annotation,
    cover_url,
    cover,
    fb2_filename,
    fb2_sha1,
    series,
    series_title
  FROM books
  WHERE title LIKE replace('%' || $1 || '%', ' ', '%')
  "#, text).fetch_all(pool)
    .await.context("db::search_book")?;

  Ok(res)
}


pub async fn get_stats(pool: &SqlitePool) -> Result<Stats> {
  let row = sqlx::query_as!(Stats, r#"
  SELECT DISTINCT
      CAST(COUNT(books.id) OVER () AS bigint) as total_books,
      CAST(COUNT(books.id) FILTER (WHERE fb2_filename IS NOT NULL) OVER () AS BIGINT) as total_fb2
  FROM books
  "#).fetch_optional(pool)
    .await.context("db::get_stats")?;
  let res = row.ok_or_else(|| Error::msg("Stats not found"))?;
  Ok(res)
}

pub async fn clear_authors(pool: &SqlitePool) -> Result<()> {
  let _row = sqlx::query_as!(Stats, r#"
  UPDATE authors SET
    books_list_fetched=false
  "#).fetch_optional(pool)
    .await.context("db::clear_authors")?;
  // let res = row.ok_or_else(|| Error::msg("Could not clear authors"))?;
  Ok(())
}
