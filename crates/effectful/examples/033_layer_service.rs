//! Ex 033 — `Layer::succeed` builds a service layer.
use effectful::{Layer, MissingService, Service, run_blocking};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Id {
  value: u64,
}

fn main() {
  let layer = Layer::<Id, MissingService>::succeed(Id { value: 99 });
  let context = run_blocking(layer.build(), ()).expect("build");
  assert_eq!(context.get::<Id>().map(|id| id.value), Some(99));
  println!("033_layer_service ok");
}
