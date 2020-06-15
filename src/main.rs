use chrono::prelude::{Local, TimeZone};
use chrono::Duration;
use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::thread;

/// Time format
const FORMAT: &str = "%Y-%m-%d %H:%M:%S%.3f";

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

/// Tweet at time
///
/// * schedule - schedule data
/// * token - access token
fn time_tweet(_schedule: Schedule, _token: &Token) {}

fn main() {
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
    time_tweet(i, &token);
  }
}
