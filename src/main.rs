use chrono::prelude::{DateTime, Local, TimeZone};
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

/// Tweet
///
/// * message - message string
/// * token - access token
/// * delete - delete setting
/// * result - result setting
fn tweet(_message: &str, _token: &Token, _delete: bool, _result: bool) -> DateTime<Local> {
  Local::now()
}

/// Tweet at time
///
/// * schedule - schedule data
/// * token - access token
fn time_tweet(schedule: Schedule, token: &Token) {
  let target_time = Local.datetime_from_str(&schedule.date, FORMAT).unwrap();
  let test_target_time: DateTime<Local> = target_time - Duration::seconds(1);
  thread::sleep(
    test_target_time
      .signed_duration_since(Local::now())
      .to_std()
      .unwrap(),
  );
  let test_date = tweet("test", token, true, false);
  let diff = test_target_time.signed_duration_since(test_date);
  thread::sleep(
    (target_time + diff)
      .signed_duration_since(Local::now())
      .to_std()
      .unwrap(),
  );
  tweet(&schedule.message, token, false, schedule.result);
}

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
