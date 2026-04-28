//! Build a [`FigmentConfigProvider`](effectful_config::FigmentConfigProvider) via
//! [`FigmentProviderLayer`](effectful_config::FigmentProviderLayer) and [`effectful::Layer`].

use effectful::LayerBuild;
use effectful_config::{FigmentProviderLayer, config, figment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let dir = tempfile::tempdir()?;
  let path = dir.path().join("c.toml");
  std::fs::write(&path, "app_name = \"example\"\n")?;

  let layer = FigmentProviderLayer::new(figment::from_toml_file(&path));
  let provider = layer.build()?;
  let name = config::string(&provider, "app_name")?;
  println!("app_name={name}");
  Ok(())
}
