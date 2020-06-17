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

/// Get OAuth for the request
///
/// * endpoint - access destination
/// * token - access token
/// * params - parameter
fn get_request_oauth(_endpoint: &str, _token: &Token, _params: Vec<(&str, &str)>) -> String {
  "".to_string()
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
fn delete_tweet(_id: i64, _token: &Token) {}

/// Post reply
///
/// * id - status id
/// * message - message string
/// * token - access token
fn post_reply(_id: i64, _message: &str, _token: &Token) {}

/// Tweet
///
/// * message - message string
/// * token - access token
/// * delete - delete setting
/// * result - result setting
async fn tweet(message: &str, token: &Token, delete: bool, result: bool) -> DateTime<Local> {
  let id = post_tweet(message, token).await.unwrap();
  if delete {
    delete_tweet(id, token);
  }
  let ms = ((id >> 22) + 1288834974657) as i64;
  let date = Local.timestamp(ms / 1000, ((ms % 1000) * 1_000_000) as u32);
  if result {
    post_reply(id, &date.format(FORMAT).to_string(), token);
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
