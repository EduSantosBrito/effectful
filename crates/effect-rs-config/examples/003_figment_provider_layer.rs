//! Build a [`FigmentConfigProvider`](effect_config::FigmentConfigProvider) via
//! [`FigmentProviderLayer`](effect_config::FigmentProviderLayer) and [`effect::Layer`].

use effect::Layer;
use effect_config::{FigmentProviderLayer, config, figment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let dir = tempfile::tempdir()?;
  let path = dir.path().join("c.toml");
  std::fs::write(&path, "app_name = \"example\"\n")?;

  let layer = FigmentProviderLayer::new(figment::from_toml_file(&path));
  let provider = Layer::build(&layer)?;
  let name = config::string(&provider, "app_name")?;
  println!("app_name={name}");
  Ok(())
}
