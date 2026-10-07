mod config;

use tokio;
use config::Config;

#[tokio::main]
async fn main() {
  let config = Config::parse().unwrap();
  
}
