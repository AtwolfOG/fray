use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use rand::{RngExt, distr::Alphanumeric, seq::IndexedRandom};
use tokio::io::{ AsyncBufReadExt, AsyncRead, BufReader};

pub async fn get_headers<T: AsyncRead + Unpin>(reader: &mut BufReader<T>) -> Result<(HashMap<String, String>, Vec<u8>)> {
  let mut buffer = Vec::new();
  loop {
    let available = reader.fill_buf().await.with_context(|| "Failed to read from buffer")?;
    if available.is_empty() {
      bail!("Unable to read header");
    }
    buffer.extend_from_slice(available);
    let amt = available.len();
    reader.consume(amt);
    if buffer.windows(4).any(|w| w == b"\r\n\r\n"){
      break;
    }
  }
  let host = extract_headers(&buffer);
  Ok((host, buffer))
}


// this extracts headers from a byte slice
fn extract_headers(headers: &[u8]) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for header in headers.split(|&b| b == b"\n"[0]) {
    let Ok(header) = str::from_utf8(header) else {
      continue;
    };
    if header.is_empty() {
      break;
    }
    if let Some(idx) = header.find(":") {
      let key = &header[..idx].trim().to_lowercase();
      let value = &header[idx + 1..].trim();
      map.insert(key.to_string(), value.to_string());
    }

  }
  map
}

const ADJECTIVES: [&str; 100] = ["red", "blue", "green", "yellow", "purple", "orange", "gold", "silver", "white", "black",
  "brave", "calm", "charming", "clever", "cool", "crazy", "curious", "dark", "daring", "dazzling",
  "eager", "fancy", "fast", "fierce", "funny", "gentle", "glorious", "graceful", "happy", "harsh",
  "honest", "jolly", "kind", "lazy", "light", "lively", "lucky", "luminous", "magical", "mighty",
  "mild", "moody", "mysterious", "noble", "noisy", "odd", "painful", "peaceful", "playful", "polite",
  "proud", "quick", "quiet", "rare", "restless", "rough", "royal", "shiny", "shy", "silent",
  "smart", "smooth", "soft", "sparkling", "speedy", "spicy", "spiky", "stable", "stealthy", "stern",
  "strong", "sturdy", "sunny", "sweet", "swift", "tame", "tender", "tough", "tranquil", "unique",
  "vague", "vast", "vibrant", "wild", "witty", "warm", "young", "zesty", "zealous", "bold",
  "ancient", "bright", "elegant", "radiant", "epic", "fearless", "gritty", "hardy", "keen", "nimble",];

const NOUNS: [&str; 100] = ["lion", "tiger", "bear", "wolf", "fox", "deer", "hawk", "eagle", "owl", "cat",
  "dog", "rabbit", "panda", "koala", "whale", "shark", "dolphin", "horse", "zebra", "rhino",
  "giraffe", "monkey", "chimp", "otter", "boar", "crocodile", "elephant", "falcon", "goose", "goat",
  "iguana", "jaguar", "kangaroo", "lemming", "leopard", "llama", "lynx", "moose", "mule", "narwhal",
  "ocelot", "octopus", "ostrich", "panther", "parrot", "penguin", "porcupine", "puma", "quokka", "raccoon",
  "rat", "salamander", "seal", "sheep", "skunk", "sloth", "squid", "swan", "tapir", "termite",
  "toad", "turkey", "turtle", "vulture", "walrus", "weasel", "yak", "aardvark", "antelope", "badger",
  "bat", "beetle", "bison", "canary", "cheetah", "crab", "donkey", "duck", "ferret", "flamingo",
  "gecko", "gopher", "hamster", "hedgehog", "heron", "hyena", "maggot", "meerkat", "mole", "monarch",
  "mongoose", "moth", "newt", "orca", "platypus", "puppy", "quail", "reindeer", "rooster", "sparrow",
];


pub fn generate_host() -> String {
  let adj = ADJECTIVES.choose(&mut rand::rng()).unwrap();
  let noun = NOUNS.choose(&mut rand::rng()).unwrap();
  format!("{}-{}", adj, noun)
}

pub fn generate_random_string(length: usize) -> String {
  let rand_str: String = rand::rng().sample_iter(&Alphanumeric)
  .take(length)
  .map(char::from)
  .collect();

  rand_str
}