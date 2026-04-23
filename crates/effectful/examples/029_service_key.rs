//! Ex 029 — `#[derive(Service)]` declares a self-describing service type.
use effectful::Service;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
pub struct ApiKey;

fn main() {
  let _ = std::any::TypeId::of::<ApiKey>();
  println!("029_service_key ok");
}
