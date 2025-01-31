use anyhow::{
  Result,
  // Error,
  Context,
};
use translit::{Gost779B, ToLatin, Language};

use crate::fetcher;
use crate::parser;
use crate::db;
use crate::filedb;


pub async fn book_page_cached(
  sqlxpool: &sqlx::sqlite::SqlitePool,
  num: u64,
) -> Result<(db::Book, parser::BookInfo, db::Author)> {
  let path = format!("/b/{}", num);
  let dbbook = db::get_book(sqlxpool, num as i64).await.ok();
  let (dbbook, book_info, dbauthor) = match dbbook {
    // if annotation is not filled - our book is from BookInfoShort
    Some(ref dbbook @ db::Book {annotation:Some(ref a), ..}
    ) => {
      log::info!("[cache] page for book {}", num);
      let dbbook = dbbook.to_owned();
      let annotation = a.to_owned();
      let dbauthor = db::get_author(sqlxpool, dbbook.author).await?;
      // let dbauthor : Result<db::Author> = dbauthor?;
      let res : Result<parser::BookInfo> =
        Ok((dbbook.clone(), dbauthor.clone(), annotation).into());
      res.map(|x| (dbbook, x, dbauthor))
    },
    // if no dbbook or dbbook without annotation - do upsert
    _ => {
      log::info!("fetching page for book {}", num);
      let html = fetcher::book(&fetcher::DEFAULT_CONF, path).await?;
      // let url = fetcher::base_url(&fetcher::DEFAULT_CONF);
      let book_info = parser::book_info(num, html).context("parser")?;
      let book : db::Book = book_info.clone().try_into()?;
      let author : db::Author = book_info.clone().try_into()?;
      let author = db::add_author(sqlxpool, author).await?;
      let dbbook = db::add_book(sqlxpool, book).await?;
      // let dbbook = db::get_book(sqlxpool, num as i64).await?;
      Ok((dbbook, book_info, author))
    },
  }?;
  Ok((dbbook, book_info, dbauthor))
}

pub async fn book_cover_cached(
  sqlxpool: &sqlx::sqlite::SqlitePool,
  dbbook: db::Book,
) -> Result<Option<bytes::Bytes>> {
  let num = dbbook.id;
  let cover = match dbbook.cover {
    Some(x) => {
      log::info!("[cache] cover for book {}", num);
      Some(bytes::Bytes::from(x))
    },
    None => match dbbook.cover_url {
      None => None,
      Some(cover_url) => {
        log::info!("fetching cover for book {}", num);
        let cover_data = fetcher::cover_image(
          &fetcher::DEFAULT_CONF, cover_url).await?;
        let _ = db::save_book_cover(sqlxpool, dbbook.id, cover_data.clone()).await?;
        Some(cover_data)
      }
    }
  };
  Ok(cover)
}

async fn fetch_fb2(
    sqlxpool: &sqlx::sqlite::SqlitePool,
    id: i64, fb2_url: String, title: String,
    author: &db::Author) -> Result<(String, bytes::Bytes)> {
    log::info!("fetching fb2 for book {}", id);
    let (fb2, filename) =
        fetcher::fb2(&fetcher::DEFAULT_CONF, fb2_url).await?;
    let fb2_sha1 = filedb::fb2_sha1(fb2.clone())?;
    log::info!("DBG: fetching fb2 for book {}: sha1 = {}", id, fb2_sha1);
    let filename = filename.unwrap_or_else(|| {
        let db::Author { name, .. } = author;
        let author = Gost779B::new(Language::Ru).to_latin(name);
        let title = Gost779B::new(Language::Ru).to_latin(&title);
        format!("{}_{}_{}.zip", id, author, title)
    });
    filedb::put_file(fb2_sha1.clone(), fb2.clone())?;
    let _ = db::save_book_fb2(
        sqlxpool, id, filename.clone(), fb2_sha1).await?;
    Ok((filename, fb2))
}


pub async fn book_fb2_cached(
    sqlxpool: &sqlx::sqlite::SqlitePool,
    dbbook: db::Book, author: db::Author
) -> Result<(String, bytes::Bytes)> {
    let fb2 = match dbbook {
        db::Book {id, fb2_sha1:Some(fb2_sha1),
                  fb2_filename:Some(fname),
                  fb2_url, title, ..} => {
            log::info!("[cache] fb2 for book {}", id);
            let fb2 = filedb::get_file(fb2_sha1);
            match fb2 {
                Err(_) => {
                    log::warn!("[cache] cachemiss for book {}", id);
                    let res : Result<(String, bytes::Bytes)> = fetch_fb2(
                        sqlxpool, id, fb2_url, title, &author).await;
                    res
                },
                Ok(fb2) => {
                    let res : Result<(String, bytes::Bytes)> = Ok((fname, fb2));
                    res
                }
            }
        },
        db::Book {id, fb2_url, title, ..} => {
            let (filename, fb2) = fetch_fb2(
                sqlxpool, id, fb2_url, title, &author).await?;
            Ok((filename, fb2))
        }
    }?;

    Ok(fb2)
}


pub async fn author_page_cached(
  sqlxpool: &sqlx::sqlite::SqlitePool,
  num: u64,
) -> Result<(bool, db::Author, parser::AuthorInfo)> {
  let path = format!("/a/{}", num);
  let dbauthor = db::get_author(sqlxpool, num as i64).await.ok();
  let (c, dbauthor, author_info) = match dbauthor {
    Some(ref dbauthor @ db::Author {books_list_fetched: true, ..}) => {
      log::info!("[cache] page for author {}", num);
      let books = db::get_books_for_author(sqlxpool, dbauthor.id).await?;
      let res : parser::AuthorInfo = (dbauthor.clone(), books).into();
      let res : Result<(bool, db::Author, parser::AuthorInfo)> =
        Ok((true, dbauthor.clone(), res));
      res
    },
    _ => {
      log::info!("fetching page for author {}", num);
      let author_id = num as i64;
      let html = fetcher::author(&fetcher::DEFAULT_CONF, path).await?;
      let author_info = parser::author_info(num, html).context("parser")?;
      let mut dbauthor : db::Author = author_info.clone().into();
      dbauthor.books_list_fetched = true;
      let dbauthor = db::add_author(sqlxpool, dbauthor).await?;
      let mut books : Vec<db::Book> = Vec::new();
      for book in author_info.books {
        let book = db::add_book(sqlxpool, (author_id, book).into()).await?;
        books.push(book);
      };
      // lets return cached (may be more filled?)
      let res : parser::AuthorInfo = (dbauthor.clone(), books).into();
      let res : Result<(bool, db::Author, parser::AuthorInfo)> =
        Ok((false, dbauthor, res));
      res
    }
  }?;

  Ok((c, dbauthor, author_info))
}


pub async fn search_cached(
  sqlxpool: &sqlx::sqlite::SqlitePool,
  text: &String,
) -> Result<Vec<(String, String)>> {
  let dbauthors = db::search_author(sqlxpool, text).await?;
  let mut results : Vec<(String, String)> = dbauthors.iter()
    .filter_map(|elt| match elt {
      db::Author {id, name, ..} => {
        let author_id = format!("/author {}", id);
        let title = name.to_owned();
        Some((title, author_id))
      },
    })
    .take(15)
    .enumerate()
    .map(|(i, (t, b))| (format!("{}. {}", i, t), b))
    .collect();
  let dbbooks = db::search_book(sqlxpool, text).await?;
  let results2 : Vec<(String, String)> = dbbooks.iter()
    .filter_map(|elt| match elt {
      db::Book {id, title, ..} => {
        let book_id = format!("/book {}", id);
        let title = title.to_owned();
        Some((title, book_id))
      },
    })
    .take(15)
    .enumerate()
    .map(|(i, (t, b))| (format!("{}. {}", i, t), b))
    .collect();
  results.extend(results2);

  Ok(results)
}


