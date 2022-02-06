use std::fmt;

use anyhow::{Result, Error};
use derive_more::{Display};
use scraper::{Html, Selector, ElementRef};
// use scraper::Selector;
use regex::Regex;

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

#[derive(Display)]
pub enum SearchResult {
  #[display(fmt = "{{author: {}}}", author)]
  Author {author: Anchor},
  #[display(fmt = "{{book: {}, author: {}}}", book, author)]
  Book {book: Anchor, author: Anchor, book_id: u64},
}

// if there are 2 links in elt - so its a book (book + author links)
// if only one - its author :)
fn find_anchor(elt: &ElementRef) -> Result<SearchResult> {
  let selector = Selector::parse("a")
    .expect("find_anchor: selector invalid");
  
  let anchors : Vec<ElementRef> = elt.select(&selector).collect();
  let res = match anchors.as_slice() {
    [author] => {
      let author : Anchor = (*author).try_into()?;
      Ok(SearchResult::Author {author})
    },
    [book, author] => {
      let author = (*author).try_into()?;
      let book : Anchor = (*book).try_into()?;
      let book_link : &String = &(book.link);
      let book_id_re = Regex::new("/b/(\\d+).+")?;
      // dbg!(&book_link);
      let book_id = book_id_re.captures_iter(&book_link).next()
        .ok_or_else(|| Error::msg("Could not find book id in link"))?
        .get(1)
        .ok_or_else(|| Error::msg("Could not find book id int in link"))?
        .as_str();
      let book_id = book_id.to_string().parse::<u64>()?;
      
      Ok(SearchResult::Book {author, book, book_id})
    },
    _ => Err(Error::msg("too many links"))
  }?;
  
  // let a = elt.select(&selector).next()
  //   .ok_or_else(|| Error::msg("find_anchor: anchor element not found"))?;
  // let a = a.try_into()?;

  Ok(res)
}


// get authors and books from search result page
pub fn search_result(html_doc: String) -> Result<Vec<SearchResult>> {
  let document = Html::parse_document(&html_doc);
  // let selector = Selector::parse("div#site-slogan")
  //   .expect("selector invalid");

  let selector = Selector::parse("div#container div#main-wrapper div#main.clear-block li")
    .expect("selector invalid");
  let selected = document.select(&selector);
  let mut result = std::vec::Vec::new();
  for row in selected {
    let row = find_anchor(&row)?;
    result.push(row);
  }

  Ok(result)
}

#[derive(Display)]
#[display(fmt = "{{title: {}, author: {}, fb2url: {}, cover: {}, annotation: \"{}\"}}", title, author, fb2url, cover, annotation)]
pub struct BookInfo {
  pub title: String,
  pub author: Anchor,
  pub fb2url: Anchor,
  pub annotation: String,
  pub cover: String,
}

pub fn book_info(html_doc: String) -> Result<BookInfo> {
  let document = Html::parse_document(&html_doc);
  let selector = Selector::parse("div#container div#main-wrapper div#main")
    .expect("selector invalid");
  let main_div = document.select(&selector)
    .next().ok_or_else(|| Error::msg("book_info: not found main div"))?;
  // searching title
  let selector = Selector::parse("h1.title")
    .expect("selector invalid");
  let selected = main_div.select(&selector)
    .next().ok_or_else(|| Error::msg("book_info: not found title"))?;

  let title = selected.text().map(|x| x.to_string())
    .collect::<Vec<String>>().join("");

  // searching author
  let selector = Selector::parse("h1.title~a[href^=\"/a/\"]")
    .expect("selector invalid");
  let author = main_div.select(&selector)
    .next().ok_or_else(|| Error::msg("book_info: not found author"))?;
  let author : Anchor = author.try_into()?;
  // author.link = &url + author.link.trim_start_matches('/');

  // searching fb2 url
  let selector = Selector::parse("h1.title~div a")
    .expect("selector invalid");
  let fb2url = main_div.select(&selector)
    .find(
      |&elt| {
        // let innertext = elt.text().map(|x| x.to_string())
        //   .collect::<Vec<String>>().join("");
        let innertext = elt.value().attr("href")
          .map(|x| x.to_string());
        // dbg!(&innertext);
        match innertext {
          None => false,
          Some(innertext) =>
            innertext.contains("/fb2")
        }
      })
    .ok_or_else(|| Error::msg("book_info: not found fb2url"))?;
  let fb2url: Anchor = fb2url.try_into()?;
  // fb2url.link = &url + fb2url.link.trim_start_matches('/');

  // searching annotation
  let selector = Selector::parse("h1.title~a[href^=\"/a/\"]~h2~p")
    .expect("selector invalid");
  let annotation = main_div.select(&selector)
    .next().ok_or_else(|| Error::msg("book_info: not found annotation"))?;
  let annotation = annotation.text().map(|x| x.to_string())
    .collect::<Vec<String>>().join("");

  // searching image cover
  let selector = Selector::parse("h1.title~img[title=\"Cover image\"]")
    .expect("selector invalid");
  let cover = main_div.select(&selector)
    .next().ok_or_else(|| Error::msg("book_info: not found image"))?;
  let cover = cover.value().attr("src")
    .map(|x| x.to_string())
    .ok_or_else(|| Error::msg("book_info: image dont have src"))?;

  let result = BookInfo {title, author, fb2url, annotation, cover};
    
  Ok(result)
}


// #[derive(Display)]
// #[display(fmt = "{{title: {}, author: {}, fb2url: {}, cover: {}, annotation: \"{}\"}}", title, author, fb2url, cover, annotation)]
// pub struct BookInfoFilled {
//   pub title: String,
//   pub author: Anchor,
//   pub fb2url: Anchor,
//   pub annotation: String,
//   pub cover: String,
//   pub cover_data: bytes::Bytes,
// }


// pub fn fill_bookinfo(bookinfo: &BookInfo, cover_data: bytes::Bytes)
//                      -> BookInfoFilled {
//   let BookInfo {title, author, fb2url
// }

