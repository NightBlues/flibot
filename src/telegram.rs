use anyhow::{Result, Error};
use teloxide::{prelude2::*, utils::command::BotCommand};
use teloxide::types::{
  InputFile,
  // InputMedia,
  // InputMediaPhoto,
  // InputMediaDocument,
  ParseMode,
  InlineKeyboardMarkup,
  InlineKeyboardButton,
};
use teloxide::utils::markdown::{
  escape as mdescape,
};

use crate::fetcher;
use crate::parser;
// use crate::db;
use crate::cache;


#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
  #[command(description = "display this text.")]
  Start,
  #[command(description = "show author.")]
  Author(u64),
  #[command(description = "show a book.")]
  Book(u64),
  #[command(description = "download a book.")]
  Download(u64),
  #[command(description = "handle a search.", separator = ";")]
  Search(String)
  // #[command(description = "handle a username and an age.", parse_with = "split")]
  // UsernameAndAge { username: String, age: u8 },
}

    // use crate::{
    //     payloads::{self, setters::*},
    //     types::{
    //         InputFile, InputMedia, InputMediaAnimation, InputMediaAudio, InputMediaDocument,
    //         InputMediaPhoto, InputMediaVideo, InputSticker, MessageEntity, MessageEntityKind,
    //         ParseMode,
    //     },
    // };


    // async fn test_send_media_group() {
    //     const CAPTION: &str = "caption";

    //     to_form_ref(&payloads::SendMediaGroup::new(
    //         0,
    //         [
    //             InputMedia::Photo(
    //                 InputMediaPhoto::new(InputFile::file("./media/logo.png"))
    //                     .caption(CAPTION)
    //                     .parse_mode(ParseMode::MarkdownV2)
    //                     .caption_entities(entities()),
    //             ),
    //             InputMedia::Video(
    //                 InputMediaVideo::new(InputFile::file_id("17")).supports_streaming(true),
    //             ),
    //             InputMedia::Animation(
    //                 InputMediaAnimation::new(InputFile::read(
    //                     File::open("./media/example.gif").await.unwrap(),
    //                 ))
    //                 .thumb(InputFile::read(
    //                     File::open("./media/logo.png").await.unwrap(),
    //                 ))
    //                 .duration(17),
    //             ),
    //             InputMedia::Audio(
    //                 InputMediaAudio::new(InputFile::url("https://example.com".parse().unwrap()))
    //                     .performer("a"),
    //             ),
    //             InputMedia::Document(InputMediaDocument::new(InputFile::memory(
    //                 &b"Hello world!"[..],
    //             ))),
    //         ],
    //     ))
    //     .unwrap()
    //     .await;
    // }


fn bookinfo_md(bookinfo: &parser::BookInfo) -> String {
  let base_url = fetcher::base_url(&fetcher::DEFAULT_CONF);
  let full_link = mdescape(&*format!("{}b/{}", base_url, bookinfo.id));
  format!(r"
*{author} \- {title}*

{annotation}
",
// [Download fb2]({fb2url})
          author=mdescape(&bookinfo.author.title),
          title=mdescape(&bookinfo.title),
          annotation=mdescape(&bookinfo.annotation),
          // fb2url=mdescape(&bookinfo.fb2url.link),
  ).chars().take(1024 - full_link.len())
    .collect::<String>() + &full_link
}


async fn book_cmd_handler(
  bot: AutoSend<Bot>,
  message: Message,
  num: u64,
  sqlxpool: &sqlx::sqlite::SqlitePool,
) -> Result<()> {
  let (dbbook, book_info) = cache::book_page_cached(sqlxpool, num).await?;
  let text = bookinfo_md(&book_info);
  let cover = cache::book_cover_cached(sqlxpool, dbbook).await?;
  log::info!("repling for book {}", num);
  let cover = cover.map(InputFile::memory);
  // responding
  let buttons : Vec<Vec<InlineKeyboardButton>> = vec![
    vec![
      InlineKeyboardButton::callback(
        "Download".to_string(),
        format!("/download {}", num)
      )
    ]
  ];
  let keyboard = InlineKeyboardMarkup::new(buttons);
 
  let _ = match cover {
    None =>
      bot.parse_mode(ParseMode::MarkdownV2)
      .send_message(message.chat.id, text).await?,
    Some(cover) => {
      // dbg!("downloaded cover {}", book_info.cover);
      bot.parse_mode(ParseMode::MarkdownV2)
        .send_photo(message.chat.id, cover)
        .caption(text)
        .reply_markup(keyboard)
        .await?
    }
  };
  // let media = [
  //   InputMedia::Photo(
  //     InputMediaPhoto::new(cover)
  //       .caption(text)
  //       .parse_mode(ParseMode::MarkdownV2)
  //   )
  // ].into_iter();
  // bot.parse_mode(ParseMode::MarkdownV2)
  //   .send_media_group(message.chat.id, media)
  //   .reply_to_message_id(message.chat.id)
  //   .await?

  Ok(())
}

async fn download_cmd_handler(
  bot: AutoSend<Bot>,
  message: Message,
  num: u64,
  sqlxpool: &sqlx::sqlite::SqlitePool,
) -> Result<()> {
  
  let (dbbook, book_info) = cache::book_page_cached(sqlxpool, num).await?;
  let (filename, fb2) = cache::book_fb2_cached(sqlxpool, dbbook.clone()).await?;
  let fb2 = InputFile::memory(fb2).file_name(filename);
  let description = format!(
    r"*{author} \- {title}*",
    author=mdescape(&book_info.author.title),
    title=mdescape(&book_info.title));
  log::info!("repling for book {}", num);
  bot.parse_mode(ParseMode::MarkdownV2)
    .send_document(message.chat.id, fb2)
    .caption(description)
    .await?;

  Ok(())
}



async fn answer(
  bot: AutoSend<Bot>,
  message: Message,
  command: Command,
  sqlxpool: &sqlx::sqlite::SqlitePool,
) -> Result<()> {
  match command {
    Command::Start => {
      bot.send_message(message.chat.id, Command::descriptions())
        .await?;
    },
    Command::Author(num) => {
      let (cached,_dbauthor, author_info) =
        cache::author_page_cached(sqlxpool, num).await?;
      // let text = (&author_info).to_string();
      let cached = if cached {
        mdescape("[cached] ")
      } else {
        "".to_string()
      };
      let base_url = fetcher::base_url(&fetcher::DEFAULT_CONF);
      let full_link = mdescape(&*format!("{}a/{}", base_url, num));
      let text = format!("{}{}", cached, &author_info)
        .chars().take(4096 - full_link.len())
        .collect::<String>() + &full_link;
      log::info!("repling for author {}", num);
      let buttons : Vec<Vec<InlineKeyboardButton>> =
        author_info.books.iter()
        .enumerate().collect::<Vec<_>>()
        .chunks(2)
        .map(|row|
             row.iter()
             .map(|(i, parser::BookInfoShort {title, book_id, ..})| {
               let t = format!("{} - {}", i, mdescape(title));
               let b = format!("/book {}", book_id);
               InlineKeyboardButton::callback(t, b)
             }).collect())
        .collect();
      let keyboard = InlineKeyboardMarkup::new(buttons);        
      bot.parse_mode(ParseMode::MarkdownV2)
        .send_message(message.chat.id, text)
        .reply_markup(keyboard)
        .await?;
    },
    Command::Book(num) => {
      book_cmd_handler(bot, message, num, sqlxpool).await?;
    },
    Command::Download(num) => {
      download_cmd_handler(bot, message, num, sqlxpool).await?;
    },
    Command::Search(text) => {
      log::info!("fetching search page for \"{}\"", &text);
      if text.trim().is_empty() {
        bot.send_message(message.chat.id, "empty request").await?;
        return Err(Error::msg("empty request"))
      }
      let html = fetcher::search(&fetcher::DEFAULT_CONF, text.clone()).await?;
      let found = parser::search_result(html)?;
      let results : Vec<(String, String)> = found.iter()
        .filter_map(|elt| match elt {
          parser::SearchResult::Author {author, author_id} => {
            let author_id = format!("/author {}", author_id);
            let title = author.title.to_owned();
            Some((title, author_id))
          },
          parser::SearchResult::Book {book, author, book_id} => {
            let title = format!("{} - {}",
                                book.title, author.title);
            let book_id = format!("/book {}", book_id);
            Some((title, book_id))
          }
        })
        .take(15)
        .enumerate()
        .map(|(i, (t, b))| (format!("{}. {}", i, t), b))
        .collect();
      let buttons : Vec<Vec<InlineKeyboardButton>> = results.chunks(2)
        .map(|row|
             row.iter()
             .map(|(title, book_id)|
                  InlineKeyboardButton::callback(
                    title.to_owned(), book_id.to_owned()))
             .collect())
        // .take(4)
        .collect();
      let keyboard = InlineKeyboardMarkup::new(buttons);
      let response = format!(
        "Found results for query \"{}\":\n{}",
        text,
        results.into_iter()
          .map(|(t,b)| format!("{} (command {})", t, b))
          .collect::<Vec<String>>().join("\n"));
      // dbg!(&response);
      log::info!("repling for search \"{}\"", text);
      bot.send_message(message.chat.id, response)
        .reply_markup(keyboard).await?;
    }
  };

  Ok(())
}

async fn message_handler(
  m: Message,
  bot: AutoSend<Bot>,
  sqlxpool: sqlx::sqlite::SqlitePool,
) -> Result<()> {
  if let Some(text) = m.text() {
    log::info!("Command: {}", text);
    match Command::parse(text, "flibot") {
      Ok(cmd) => {
        let chat_id = m.chat.id;
        let res = answer(bot.clone(), m, cmd, &sqlxpool).await;
        let res = match res {
          Ok(()) => Ok(()),
          Err(e) => {
            let stre = format!("Command error: {}", e);
            bot.send_message(chat_id, stre).await?;
            Err(e)
          }
        }?;
        res
      }
      Err(_) => {
        let e = format!("Command '{}' not found!", text);
        bot.send_message(m.chat.id, e).await?;
      }
    }
  }
  
  Ok(())
}


async fn callback_handler(
  q: CallbackQuery,
  bot: AutoSend<Bot>,
  sqlxpool: sqlx::sqlite::SqlitePool,
) -> Result<()> {
  if let Some(command) = q.data {
    log::info!("Reply command: {}", command);
    // let text = format!("You chose: {}", version);

    match q.message {
      Some(m) => {
        // message_handler(message, bot).await?;
        match Command::parse(&command, "flibot") {
          Ok(cmd) => {
            let chat_id = m.chat.id;
            let res = answer(bot.clone(), m, cmd, &sqlxpool).await;
            let res = match res {
              Ok(()) => Ok(()),
              Err(e) => {
                let stre = format!("Command error: {}", e);
                bot.send_message(chat_id, stre).await?;
                Err(e)
              }
            }?;
            res
          }
          Err(_) => {
            bot.send_message(m.chat.id, "Command not found!").await?;
          }
        }
      }
      None => {
        log::warn!("not supported inline bot");
        // if let Some(id) = q.inline_message_id {
        //   bot.edit_message_text_inline(id, text).await?;
        // }
      }
    }

  }

  Ok(())
}


pub async fn start_bot(sqlxpool : sqlx::sqlite::SqlitePool) -> Result<()> {
  // TELOXIDE_TOKEN
  teloxide::enable_logging!();
  log::info!("Starting flibot...");

  let bot = Bot::from_env().auto_send();

  // teloxide::repls2::repl(bot, |message: Message, bot: AutoSend<Bot>| async move {
  //   bot.send_dice(message.chat.id).await?;
  //   respond(())
  // })
  //   .await;

  // teloxide::repls2::commands_repl(bot, answer, Command::ty()).await;
   let handler = dptree::entry()
    .branch(Update::filter_message().endpoint(message_handler))
    .branch(Update::filter_callback_query().endpoint(callback_handler));
  Dispatcher::builder(bot, handler)
    .dependencies(dptree::deps![sqlxpool])
    .build().setup_ctrlc_handler().dispatch().await;


  Ok(())
}
