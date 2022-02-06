use anyhow::Result;
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


#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum Command {
  #[command(description = "display this text.")]
  Help,
  #[command(description = "handle a book.")]
  Book(u64),
  #[command(description = "handle a search.")]
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
  format!(r"
*{author} \- {title}*

{annotation}

[Download fb2]({fb2url})
",
          author=mdescape(&bookinfo.author.title),
          title=mdescape(&bookinfo.title),
          annotation=mdescape(&bookinfo.annotation),
          fb2url=mdescape(&bookinfo.fb2url.link))
}


async fn answer(
  bot: AutoSend<Bot>,
  message: Message,
  command: Command,
) -> Result<()> {
  match command {
    Command::Help => {
      bot.send_message(message.chat.id, Command::descriptions())
        .await?;
    },
    Command::Book(num) => {
      let path = format!("/b/{}", num);
      let html = fetcher::book(&fetcher::DEFAULT_CONF, path).await?;
      // let url = fetcher::base_url(&fetcher::DEFAULT_CONF);
      let book_info = parser::book_info(html)?;
      let text = bookinfo_md(&book_info);
      dbg!(&text);
      // teloxide::utils::markdown::bold
      // bot.parse_mode(ParseMode::MarkdownV2)
      //   .send_message(message.chat.id, resp)
      //   .await?;
      let cover_data = fetcher::cover_image(
        &fetcher::DEFAULT_CONF,
        book_info.cover).await?;
      let cover = InputFile::memory(cover_data);
      // dbg!("downloaded cover {}", book_info.cover);
      bot.parse_mode(ParseMode::MarkdownV2)
        .send_photo(message.chat.id, cover)
        .caption(text)
        .await?;
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
    }
    Command::Search(text) => {
      let html = fetcher::search(&fetcher::DEFAULT_CONF, text).await?;
      let found = parser::search_result(html)?;
      let buttons : Vec<InlineKeyboardButton> = found.into_iter().
        filter_map(|elt| match elt {
          parser::SearchResult::Author {..} => None,
          parser::SearchResult::Book {book, author, book_id} => {
            let title = format!("{} - {}", author.title, book.title);
            let book_id = format!("/book {}", book_id);
            Some(InlineKeyboardButton::callback(title, book_id))
          },
        }).collect();
      let keyboard = InlineKeyboardMarkup::new(vec![buttons]);
      bot.send_message(message.chat.id, "Found results:")
        .reply_markup(keyboard).await?;      
    }
    // Command::UsernameAndAge { username, age } => {
    //     bot.send_message(
    //         message.chat.id,
    //         format!("Your username is @{} and age is {}.", username, age),
    //     )
    //     .await?
    // }
  };

  Ok(())
}


pub async fn start_bot() -> Result<()> {
  // TELOXIDE_TOKEN
  teloxide::enable_logging!();
  log::info!("Starting flibot...");

  let bot = Bot::from_env().auto_send();

  // teloxide::repls2::repl(bot, |message: Message, bot: AutoSend<Bot>| async move {
  //   bot.send_dice(message.chat.id).await?;
  //   respond(())
  // })
  //   .await;

  teloxide::repls2::commands_repl(bot, answer, Command::ty()).await;

  Ok(())
}
