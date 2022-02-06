use anyhow::{Result, Error};

pub struct Conf<'a> {
  pub proto: &'a str,
  pub socks: &'a str,
  pub domain: &'a str,
}

pub const DEFAULT_CONF : Conf = Conf {
  proto: "http",
  socks: "socks5h://127.0.0.1:9050",
  domain: "flibustaongezhld6dibs2dps6vm4nvqg2kp7vgowbu76tzopgnhazqd.onion",
};

pub fn base_url(
  &Conf {proto, domain, ..}: &'static Conf<'static>) -> String {
  format!("{}://{}/", proto, domain)
}

async fn _get_page(
  &Conf {proto, socks, domain}: &'static Conf<'static>,
  path: String,
  query: impl Into<Option<Vec<(String, String)>>>)
  -> Result<reqwest::RequestBuilder> {
  let tor = reqwest::Proxy::all(socks)?;
  let client = reqwest::Client::builder()
    .proxy(tor)
  // .resolve(flibusta_domain, SocketAddr::from(([127, 0, 0, 1], 9050)))
    .build()?;
  let flibusta_url = format!("{}://{}/{}",
                             proto,
                             domain,
                             path.trim_start_matches('/'));
  let request = client.get(&flibusta_url)
    .header("User-Agent", "Mozilla/5.0 _X11; Linux x86_64; rv:96.0_ Gecko/20100101 Firefox/96.0");
  let request = match query.into() {
    None => request,
    Some(query) => request.query(&query),
  };
  Ok(request)
}

pub async fn get_page(
  conf: &'static Conf<'static>,
  path: String,
  query: impl Into<Option<Vec<(String, String)>>>)
  -> Result<String> {
  let request = _get_page(conf, path, query).await?;

  let resp = request.send().await?;
  let page = resp.text().await?;

  Ok(page)
}

pub async fn get_blob(
  conf: &'static Conf<'static>,
  path: String,
  query: impl Into<Option<Vec<(String, String)>>>)
  -> Result<bytes::Bytes> {
  let request = _get_page(conf, path, query).await?;

  let resp = request.send().await?;
  let page = resp.bytes().await?;

  Ok(page)
}


// search only authors and books on flibusta (to ease parsing)
pub async fn search(conf: &'static Conf<'static>, search: String) -> Result<String> {
  let query : Vec<(String, String)> = vec![
    ("ask".into(), search),
    ("cha".into(), "on".into()),
    ("chb".into(), "on".into())
  ].into();
  let result = get_page(conf, "booksearch".into(), query).await?;
  
  Ok(result)
}


// get book details page
pub async fn book(conf: &'static Conf<'static>, path: String) -> Result<String> {
  let path = if path.starts_with("/b/") {
    Ok(path)
  } else {
    Err(Error::msg("fecher::book got invalid path"))
  }?;

  let result = get_page(conf, path, None).await?;

  Ok(result)
}

// download cover image for book
pub async fn cover_image(conf: &'static Conf<'static>, path: String) -> Result<bytes::Bytes> {
  let result = get_blob(conf, path, None).await?;

  Ok(result)
}
