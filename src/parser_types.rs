use std::fmt;
use std::fmt::Debug;

use derive_more::{Display};
use anyhow::{Result, Error};
use regex::Regex;
use scraper::{ElementRef};
use teloxide::utils::markdown::{
  escape as mdescape,
};
use itertools::Itertools;


use crate::db;


#[derive(Debug, Clone)]
pub struct Anchor {
  pub link: String,
  pub title: String,
}

impl<'a> TryFrom<ElementRef<'a>> for Anchor {
  type Error = Error;
  fn try_from(a: ElementRef) -> Result<Anchor> {
    let title = a.text().map(|x| x.to_string())
      .collect::<Vec<String>>().join("");
    let link = a.value().attr("href")
      .map(|x| x.to_string())
      .ok_or_else(|| Error::msg("find_anchor: anchor doesn't have href"))?;
    Ok(Anchor {title, link})
  }
}

impl fmt::Display for Anchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{{}: {}}}", self.link, self.title)
    }
}


pub fn book_id_from_url(book_link: &String) -> Result<u64> {
  let book_id_re = Regex::new("/b/(\\d+).*")?;
  let book_id = book_id_re.captures_iter(&book_link).next()
    .ok_or_else(|| Error::msg("Could not find book id in link"))?
    .get(1)
    .ok_or_else(|| Error::msg("Could not find book id int in link"))?
    .as_str();
  let book_id = book_id.to_string().parse::<u64>()?;
  Ok(book_id)
}

pub fn author_id_from_url(author_link: &String) -> Result<u64> {
  let author_id_re = Regex::new("/a/(\\d+).*")?;
  let author_id = author_id_re.captures_iter(&author_link).next()
    .ok_or_else(|| Error::msg("Could not find author id in link"))?
    .get(1)
    .ok_or_else(|| Error::msg("Could not find author id int in link"))?
    .as_str();
  let author_id = author_id.to_string().parse::<u64>()?;
  Ok(author_id)
}

pub fn series_id_from_url(series_link: &String) -> Result<u64> {
  let series_id_re = Regex::new("/s/(\\d+).*")?;
  let series_id = series_id_re.captures_iter(&series_link).next()
    .ok_or_else(|| Error::msg("Could not find series id in link"))?
    .get(1)
    .ok_or_else(|| Error::msg("Could not find series id int in link"))?
    .as_str();
  let series_id = series_id.to_string().parse::<u64>()?;
  Ok(series_id)
}



#[derive(Display, Clone)]
pub enum SearchResult {
  #[display(fmt = "{{author: {}}}", author)]
  Author {author: Anchor, author_id: u64},
  #[display(fmt = "{{book: {}, author: {}}}", book, author)]
  Book {book: Anchor, author: Anchor, book_id: u64},
}

#[derive(Display, Clone)]
#[display(fmt = "{{title: {}, author: {}, fb2url: {}, cover_url: {:?}, annotation: \"{}\"}}", title, author, fb2url, cover_url, annotation)]
pub struct BookInfo {
  pub id: i64,
  pub title: String,
  pub author: Anchor,
  pub fb2url: Anchor,
  pub annotation: String,
  pub cover_url: Option<String>,
  pub series: Option<(i64, String)>,
}

impl TryFrom<BookInfo> for db::Book {
  type Error = Error;
  fn try_from(
    BookInfo {id, title, author, fb2url, annotation, cover_url,
              series,}: BookInfo
  ) -> Result<db::Book> {
    // let id = book_id_from_url(&fb2url.link)? as i64;
    let author = author_id_from_url(&author.link)? as i64;
    let annotation = Some(annotation);
    let Anchor {link:fb2_url, ..} = fb2url;
    let (series, series_title) = match series {
      None => (None, None),
      Some((a, b)) => (Some(a), Some(b)),
    };
    Ok(db::Book {
      id,
      title,
      author,
      mark:None,
      annotation,
      cover_url,
      cover:None,
      fb2_url,
      fb2_filename:None,
      fb2_sha1:None,
      series,
      series_title
    })
  }
}

impl TryFrom<BookInfo> for db::Author {
  type Error = Error;
  fn try_from(
    BookInfo {author, ..}: BookInfo
  ) -> Result<db::Author> {
    let id = author_id_from_url(&author.link)? as i64;
    let Anchor {link:url, title:name} = author;
    Ok(db::Author {
      id,
      name,
      url,
      // bookinfo has no books list info
      books_list_fetched:false
    })
  }
}

impl From<(db::Book, db::Author, String)> for BookInfo {
  fn from(
    (db::Book {id, title, fb2_url, cover_url, series, series_title, ..},
     db::Author {name, url:author_url, ..},
     annotation):
    (db::Book, db::Author, String)
  ) -> BookInfo {
    let author = Anchor {link: author_url, title: name};
    let fb2url = Anchor {link: fb2_url, title: "(fb2)".to_string()};
    let series = series.zip(series_title);
    BookInfo {
      id,
      title,
      author,
      fb2url,
      annotation,
      cover_url,
      series
    }
  }
}


#[derive(Clone)]
pub struct BookInfoShort {
  pub book_id: u64,
  pub fb2_url: String,
  pub title: String,
  pub mark: Option<f64>,
  pub series: Option<(i64, String)>,
}

impl fmt::Display for BookInfoShort {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mark = self.mark.map(|x| x.to_string())
      .unwrap_or_else(|| "-".to_string());
    let title = mdescape(&self.title);
    write!(f, r#"\({}\) {} \(/book {}\)"#,
           mdescape(&mark), title, self.book_id)
  }
}

/// Getting BookInfoShort from db (from cache)
impl From<db::Book> for BookInfoShort {
  fn from(db::Book {id, title, mark, fb2_url, series, series_title, ..} : db::Book) -> BookInfoShort {
    let series = series.zip(series_title);
    BookInfoShort {
      book_id: id as u64,
      fb2_url,
      title,
      mark,
      series
    }
  }
}

/// Getting db::Book to save page result to cache
impl From<(i64, BookInfoShort)> for db::Book {
  fn from(
    (author_id,
     BookInfoShort {book_id, fb2_url, title, mark,
                    series,}
    ) : (i64, BookInfoShort)
  ) -> db::Book {
    // let (series, series_title) = series.unzip();
    let (series, series_title) = match series {
      None => (None, None),
      Some((a, b)) => (Some(a), Some(b)),
    };
    db::Book {
      id:book_id as i64,
      title,
      author:author_id,
      fb2_url,
      mark,
      annotation:None,
      cover_url:None,
      cover:None,
      fb2_sha1:None,
      fb2_filename:None,
      series,
      series_title
    }    
  }
}

#[derive(Clone)]
pub struct AuthorInfo {
  pub id: i64,
  pub author: String,
  pub books: Vec<BookInfoShort>,
}

impl AuthorInfo {
  pub fn split_parts(self, n: usize) -> Vec<AuthorInfo> {
    if self.books.len() < n {
      return vec![self];
    }
    let mut result : Vec<AuthorInfo> = vec![];
    let mut i_prev = 0;
    for i in (n..self.books.len()).step_by(n) {
      let books = self.books[i_prev..i].to_vec();
      let ai = AuthorInfo {books, .. self.clone()};
      result.push(ai);
      i_prev = i;
    }
    result
  }
}

impl fmt::Display for AuthorInfo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let AuthorInfo {author, books, ..} = self;
    // let mut books = books.clone();
    // books.sort_by_key(
    //   // hope stable sort do th magic
    //   |BookInfoShort {series, ..}|
    //   series.clone().map(|(x, _)| x).unwrap_or(0x0FFFFFFFFFFFFFFF));
    let books = &books.iter().enumerate()
      .group_by(|(_, BookInfoShort {series, ..})| series)
      .into_iter()
      .map(|(series, books)| {
        let books = books
          .map(|(i, book)| format!(r#"{}\. {}"#, i, book))
          .collect::<Vec<String>>()
          .join("\n");
        // let series = series.map(|(_, x)| *x.clone()).unwrap_or(String::new());
        let series = match series {
          None => String::new(),
          Some((_, x)) => mdescape(x) // .clone(),
        };
        format!("{}\n{}", series, books)
      })
      .collect::<Vec<String>>()
      .join("\n");      
    write!(f, r#"*{author}*
index mark title command
{books}
"#, author=mdescape(author), books=books)
  }
}

/// Create db::Author assuming we know his books index
impl From<AuthorInfo> for db::Author {
  fn from(AuthorInfo {id, author, ..} : AuthorInfo) -> db::Author {
    let url = format!("/a/{}", id);
    db::Author {
      id,
      name: author,
      url,
      // we construct db::Author having books index
      books_list_fetched:true,
    }
  }
}

/// Create db:Book(s) from AuthorInfo (create cache from page)
impl From<AuthorInfo> for Vec<db::Book> {
  fn from(AuthorInfo {id, books, ..} : AuthorInfo
  ) -> Vec<db::Book> {
    let books = books.into_iter()
      .map(|b| (id, b).into())
      .collect::<Vec<db::Book>>();
    books
  }
}

/// Create AuthorInfo from db data (getting from cache)
impl From<(db::Author, Vec<db::Book>)> for AuthorInfo {
  fn from(
    (db::Author {id, name, ..}, books)
      : (db::Author, Vec<db::Book>)
  ) -> AuthorInfo {
    let books = books.into_iter()
      .map(|b| b.into())
      .collect::<Vec<BookInfoShort>>();
    AuthorInfo {id, author:name, books}
  }
}
