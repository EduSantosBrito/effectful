//! [`OrElseConfigProvider`](effectful_config::OrElseConfigProvider): try one source, then another.

use effectful_config::config;
use effectful_config::{MapConfigProvider, OrElseConfigProvider};
use std::collections::HashMap;

fn main() -> Result<(), effectful_config::ConfigError> {
  let primary = MapConfigProvider::from_map(HashMap::new());
  let backup = MapConfigProvider::from_pairs([("API_KEY", "fallback-key")]);
  let p = OrElseConfigProvider::new(primary, backup);

  println!("{}", config::string(&p, "API_KEY")?);
  Ok(())
}
