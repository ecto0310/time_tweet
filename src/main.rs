use serde::Deserialize;
use std::fs::File;
use std::io::BufReader;

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

fn main() {
  // Load schedule
  let file = File::open("data.json").unwrap();
  let reader = BufReader::new(file);
  let data: Data = serde_json::from_reader(reader).unwrap();
  let _token = data.token;
  let _schedule = data.schedule;
}
