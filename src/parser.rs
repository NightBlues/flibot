
use anyhow::{Result, Error};

use scraper::{Html, Selector, ElementRef};
// use scraper::Selector;
use regex::Regex;

pub use crate::parser_types::{
  Anchor,
  SearchResult,
  BookInfo,
  BookInfoShort,
  AuthorInfo,
  author_id_from_url,
  book_id_from_url,
};


/// if there are 2 links in elt - so its a book (book + author links)
/// if only one - its author :)
fn find_anchor(elt: &ElementRef) -> Result<Option<SearchResult>> {
  let selector = Selector::parse("a")
    .expect("find_anchor: selector invalid");
  
  let anchors : Vec<ElementRef> = elt.select(&selector).collect();
  let res : Result<Option<SearchResult>> = match anchors.as_slice() {
    [author] => {
      let author : Anchor = (*author).try_into()?;
      let author_link = &(author.link);
      let author_id = author_id_from_url(author_link)?;
      Ok(Some(SearchResult::Author {author, author_id}))
    },
    [book, author] => {
      let author = (*author).try_into()?;
      let book : Anchor = (*book).try_into()?;
      let book_link : &String = &(book.link);
      // dbg!(&book_link);
      let book_id = book_id_from_url(book_link)?;
      
      Ok(Some(SearchResult::Book {author, book, book_id}))
    },
    [] => Ok(None),
    _ => {
      // Err(Error::msg("find_anchor: too many links"))
      let elt = elt.text().map(|x| x.to_string())
        .collect::<Vec<String>>().join("");
      log::warn!("find_anchor: too many links {{{}}}", elt);
      Ok(None)
    }
  };
  
  // let a = elt.select(&selector).next()
  //   .ok_or_else(|| Error::msg("find_anchor: anchor element not found"))?;
  // let a = a.try_into()?;

  res
}


/// get authors and books from search result page
pub fn search_result(html_doc: String) -> Result<Vec<SearchResult>> {
  let document = Html::parse_document(&html_doc);
  // let selector = Selector::parse("div#site-slogan")
  //   .expect("selector invalid");

  let selector = Selector::parse("div#container div#main-wrapper div#main.clear-block li")
    .expect("selector invalid");
  let selected = document.select(&selector);
  let mut result = std::vec::Vec::new();
  for row in selected {
    let row = find_anchor(&row);
    if let Ok(Some(row)) = row {
      result.push(row);
    }
  }

  Ok(result)
}


/// get book info from book detail page
pub fn book_info(id: u64, html_doc: String) -> Result<BookInfo> {
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
  let annotation = main_div.select(&selector).next();
  let annotation = match annotation {
    None => "[annotation not found]".to_string(),
    Some(annotation) => annotation.text().map(|x| x.to_string())
      .collect::<Vec<String>>().join(""),
  };

  // searching image cover
  let selector = Selector::parse("h1.title~img[title=\"Cover image\"]")
    .expect("selector invalid");
  let cover_url = main_div.select(&selector).next()
    .map(|cover| cover.value().attr("src")).flatten()
    .map(|x| x.to_string());

  let result = BookInfo {id:id as i64, title, author, fb2url, annotation, cover_url};
    
  Ok(result)
}


fn try_find_mark(book: &ElementRef) -> Option<f32> {
  let mut prev_elt = book.prev_sibling()?;
  for _ in 0..4 {
    let prev_elt_ = prev_elt.prev_sibling()?;
    prev_elt = prev_elt_;
    let tag = ElementRef::wrap(prev_elt).map(|x| x.value().name());
    if tag == Some("svg") {
      break;
    }
  }
  let svg = ElementRef::wrap(prev_elt)?;
  // dbg!("svg = {}", svg);
  let selector = Selector::parse("rect title")
    .expect("selector invalid");
  let res = svg.select(&selector).next()?.text().map(|x| x.to_string())
    .collect::<Vec<String>>().join("");
  let re = Regex::new(r#".*: ([0-9.]+)"#).ok()?;
  let res = re.captures(&res)?.get(1)?.as_str().to_string().parse::<f32>().ok()?;

  Some(res)
}

/// get author info from book detail page
pub fn author_info(id: u64, html_doc: String) -> Result<AuthorInfo> {
  let document = Html::parse_document(&html_doc);
  let selector = Selector::parse("div#container div#main-wrapper div#main")
    .expect("selector invalid");
  let main_div = document.select(&selector)
    .next().ok_or_else(|| Error::msg("author_info: not found main div"))?;
  // searching title
  let selector = Selector::parse("h1.title")
    .expect("selector invalid");
  let selected = main_div.select(&selector)
    .next().ok_or_else(|| Error::msg("author_info: not found title"))?;
  let author = selected.text().map(|x| x.to_string())
    .collect::<Vec<String>>().join("");

  // searching book
  let mut books = std::vec::Vec::new();
  let selector = Selector::parse(r#"svg+input[type="checkbox"]+a[href^="/b/"]"#)
    .expect("selector invalid book a");
  let selected = main_div.select(&selector);
  for book in selected {
    let mark = try_find_mark(&book);
    let url : Result<Anchor> = book.try_into();
    if let Ok(b) = url {
      let title = b.title;
      if let Ok(book_id) = book_id_from_url(&b.link) {
        // let fb2_url = b.link;
        // TODO: find real <a> with fb2 link in siblings
        let fb2_url = format!("{}/fb2", b.link);
        books.push(BookInfoShort {book_id, fb2_url, title, mark});
      }
    };
  }

  let result = AuthorInfo {id:id as i64, author, books};
    
  Ok(result)
}
