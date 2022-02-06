use anyhow::Result;

mod fetcher;
mod parser;
mod telegram;
// use fetcher::DEFAULT_CONF;

#[tokio::main]
async fn main() -> Result<()> {
  // let search = String::from("Анджей Сапковски");
  // let page = fetcher::search(DEFAULT_CONF, search).await?;
  // let page = fetcher::book(&DEFAULT_CONF, "/b/577468".into()).await?;
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

  telegram::start_bot().await?;
  
  Ok(())
}
