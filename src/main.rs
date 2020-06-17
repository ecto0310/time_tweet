use chrono::prelude::{DateTime, Local, TimeZone};
use chrono::Duration;
use percent_encoding::{utf8_percent_encode, AsciiSet};
use reqwest::header::HeaderMap;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::thread;

/// Time format
const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

/// Conversion rule
const FRAGMENT: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
  .remove(b'*')
  .remove(b'-')
  .remove(b'.')
  .remove(b'_')
  .remove(b'~');

/// Key and token structure for calling API
#[derive(Debug, Deserialize)]
struct Token {
  consumer_key: String,
  consumer_secret: String,
  oauth_token: String,
  oauth_token_secret: String,
}

/// Schedule structure
#[derive(Debug, Deserialize)]
struct Schedule {
  date: String,
  message: String,
  result: bool,
}

/// Configuration structure
#[derive(Debug, Deserialize)]
struct Data {
  token: Token,
  schedule: Vec<Schedule>,
}

/// Response structure
#[derive(Debug, Deserialize)]
struct Response {
  id: i64,
}

/// Get signature for the oauth
///
/// * http_method - http method
/// * endpoint - access destination
/// * token - access token
/// * param - parameter
fn get_oauth_signature(
  http_method: &str,
  endpoint: &str,
  token: &Token,
  params: Vec<(&str, &str)>,
) -> String {
  let key = format!(
    "{}&{}",
    utf8_percent_encode(&token.consumer_secret, FRAGMENT),
    utf8_percent_encode(&token.oauth_token_secret, FRAGMENT)
  );

  let mut params = params;
  params.sort();
  let params = params
    .into_iter()
    .map(|(k, v)| {
      format!(
        "{}={}",
        utf8_percent_encode(k, FRAGMENT),
        utf8_percent_encode(v, FRAGMENT)
      )
    })
    .collect::<Vec<String>>()
    .join("&");

  let http_method = utf8_percent_encode(http_method, FRAGMENT);
  let endpoint = utf8_percent_encode(endpoint, FRAGMENT);
  let param = utf8_percent_encode(&params, FRAGMENT);

  let data = format!("{}&{}&{}", http_method, endpoint, param);

  let hash = hmacsha1::hmac_sha1(key.as_bytes(), data.as_bytes());
  base64::encode(hash)
}

/// Get OAuth for the request
///
/// * endpoint - access destination
/// * token - access token
/// * params - parameter
fn get_request_oauth(endpoint: &str, token: &Token, params: Vec<(&str, &str)>) -> String {
  let oauth_nonce = &format!("{}", Local::now().timestamp());
  let oauth_signature_method = "HMAC-SHA1";
  let oauth_timestamp = &format!("{}", Local::now().timestamp());
  let oauth_version = "1.0";

  let mut params = params;
  params.push(("oauth_consumer_key", &token.consumer_key));
  params.push(("oauth_nonce", oauth_nonce));
  params.push(("oauth_signature_method", oauth_signature_method));
  params.push(("oauth_timestamp", oauth_timestamp));
  params.push(("oauth_token", &token.oauth_token));
  params.push(("oauth_version", oauth_version));

  let oauth_signature = &get_oauth_signature("POST", &endpoint, &token, params);

  format!(
    "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"{}\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"{}\"",
    utf8_percent_encode(&token.consumer_key, FRAGMENT),
    utf8_percent_encode(oauth_nonce, FRAGMENT),
    utf8_percent_encode(oauth_signature, FRAGMENT),
    utf8_percent_encode(oauth_signature_method, FRAGMENT),
    utf8_percent_encode(oauth_timestamp, FRAGMENT),
    utf8_percent_encode(&token.oauth_token, FRAGMENT),
    utf8_percent_encode(oauth_version, FRAGMENT),
  )
}

/// Post tweet
///
/// * message - message string
/// * token - access token
async fn post_tweet(message: &str, token: &Token) -> Result<i64, reqwest::Error> {
  let endpoint = "https://api.twitter.com/1.1/statuses/update.json";
  let mut params: Vec<(&str, &str)> = Vec::new();
  params.push(("status", message));
  let mut headers = HeaderMap::new();
  headers.insert(
    "Authorization",
    get_request_oauth(endpoint, token, params).parse().unwrap(),
  );
  headers.insert(
    "Content-Type",
    "application/x-www-form-urlencoded".parse().unwrap(),
  );

  let res: Response = reqwest::Client::new()
    .post(endpoint)
    .headers(headers)
    .body(format!("status={}", utf8_percent_encode(message, FRAGMENT)))
    .send()
    .await?
    .json()
    .await?;
  Ok(res.id)
}

/// Delete tweet
///
/// * id - status id
/// * token - access token
async fn delete_tweet(id: i64, token: &Token) -> Result<(), reqwest::Error> {
  let endpoint = &format!("https://api.twitter.com/1.1/statuses/destroy/{}.json", id);
  let mut headers = HeaderMap::new();
  headers.insert(
    "Authorization",
    get_request_oauth(endpoint, token, Vec::<(&str, &str)>::new())
      .parse()
      .unwrap(),
  );

  reqwest::Client::new()
    .post(endpoint)
    .headers(headers)
    .send()
    .await?;
  Ok(())
}

/// Post reply
///
/// * id - status id
/// * message - message string
/// * token - access token
async fn post_reply(id: i64, message: &str, token: &Token) -> Result<(), reqwest::Error> {
  let endpoint = "https://api.twitter.com/1.1/statuses/update.json";
  let mut params: Vec<(&str, &str)> = Vec::new();
  params.push(("status", message));
  let id = &id.to_string();
  params.push(("in_reply_to_status_id", id));
  let mut headers = HeaderMap::new();
  headers.insert(
    "Authorization",
    get_request_oauth(endpoint, token, params).parse().unwrap(),
  );
  headers.insert(
    "Content-Type",
    "application/x-www-form-urlencoded".parse().unwrap(),
  );

  reqwest::Client::new()
    .post(endpoint)
    .headers(headers)
    .body(format!(
      "status={}&in_reply_to_status_id={}",
      utf8_percent_encode(message, FRAGMENT),
      utf8_percent_encode(id, FRAGMENT)
    ))
    .send()
    .await?;
  Ok(())
}

/// Tweet
///
/// * message - message string
/// * token - access token
/// * delete - delete setting
/// * result - result setting
async fn tweet(message: &str, token: &Token, delete: bool, result: bool) -> DateTime<Local> {
  let id = post_tweet(message, token).await.unwrap();
  if delete {
    delete_tweet(id, token).await.unwrap();
  }
  let ms = ((id >> 22) + 1288834974657) as i64;
  let date = Local.timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32);
  if result {
    post_reply(id, &date.format(FORMAT).to_string(), token)
      .await
      .unwrap();
  }
  date
}

/// Tweet at time
///
/// * schedule - schedule data
/// * token - access token
async fn time_tweet(schedule: Schedule, token: &Token) {
  let target_time = Local.datetime_from_str(&schedule.date, FORMAT).unwrap();
  let test_target_time: DateTime<Local> = target_time - Duration::seconds(1);
  thread::sleep(
    test_target_time
      .signed_duration_since(Local::now())
      .to_std()
      .unwrap(),
  );
  let test_date = tweet("test", token, true, false).await;
  let diff = test_target_time.signed_duration_since(test_date);
  thread::sleep(
    (target_time + diff)
      .signed_duration_since(Local::now())
      .to_std()
      .unwrap(),
  );
  tweet(&schedule.message, token, false, schedule.result).await;
}

#[tokio::main]
async fn main() -> Result<(), ()> {
  // Load schedule
  let file = File::open("data.json").unwrap();
  let reader = BufReader::new(file);
  let data: Data = serde_json::from_reader(reader).unwrap();
  let token = data.token;
  let schedule = data.schedule;

  // tweet
  for i in schedule {
    let target_time = Local.datetime_from_str(&i.date, FORMAT).unwrap();
    let wait = (target_time - Duration::seconds(2))
      .signed_duration_since(Local::now())
      .to_std()
      .unwrap();
    thread::sleep(wait);
    time_tweet(i, &token).await;
  }
  Ok(())
}
