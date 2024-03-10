use anyhow::Result;
use sqlx::sqlite::{
  SqlitePoolOptions,
  // SqliteConnectOptions,
};

mod db;
mod fetcher;
mod parser_types;
mod parser;
mod cache;
mod telegram;
// use fetcher::DEFAULT_CONF;

#[tokio::main]
async fn main() -> Result<()> {
  // let search = String::from("Анджей Сапковски");
  // let page = fetcher::search(&fetcher::DEFAULT_CONF, search).await?;
  // let page = fetcher::book(&fetcher::DEFAULT_CONF, "/b/577468".into()).await?;
  // let page = fetcher::author(&fetcher::DEFAULT_CONF, "/a/107253".into()).await?;
  // std::fs::write("/tmp/out2", &page)?;
  // let page = std::fs::read_to_string("/tmp/out2")?;
  // println!("{}", page);

  // let found = parser::search_result(page)?;
  // let found = found.into_iter().map(|x| x.to_string())
  //   .collect::<Vec<String>>()
  //   .join("\n");
  // println!("Found results:\n{}", found);

  // let bookinfo = parser::book_info(page)?;
  // println!("Bookinfo:\n{}", bookinfo);

  // let authorinfo = parser::author_info(107253, page)?;
  // println!("Authorinfo:\n{}", authorinfo);


  // pretty_env_logger::init_timed();
  pretty_env_logger::formatted_timed_builder()
    .write_style(pretty_env_logger::env_logger::WriteStyle::Auto)
    .filter(Some("flibot"), log::LevelFilter::Info)
    .filter(Some("teloxide"), log::LevelFilter::Info)
    .init();
  let database_url = std::env::var("DATABASE_URL")
    .expect("Specify DATABASE_URL env var.");
  // let conn_str : SqliteConnectOptions = database_url.parse()?;
  let sqlxpool = SqlitePoolOptions::new()
      // .max_connections(10)
    .connect(&*database_url).await?;

  sqlx::migrate!().run(&sqlxpool).await?;

  let admins : Vec<String> = std::env::var("ADMINS")
    .map(|x| x.split(",").map(|x| x.to_string()).collect())
    .unwrap_or(vec![]);

  // uncomment to start)
  telegram::start_bot(sqlxpool, admins).await?;
  
  Ok(())
}
