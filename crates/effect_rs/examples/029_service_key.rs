//! Ex 029 — `service_key!` declares a nominal tag type.
use effect_rs::service_key;

service_key!(pub struct ApiKey);

fn main() {
  let _ = std::any::TypeId::of::<ApiKey>();
  println!("029_service_key ok");
}
