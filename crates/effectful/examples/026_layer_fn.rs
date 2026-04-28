//! Ex 026 — `LayerFn` builds one layer cell.
use effectful::{LayerBuild, LayerFn, Service, Tagged};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct SeedKey;

fn main() {
  let layer = LayerFn(|| Ok::<Tagged<SeedKey, u32>, ()>(Tagged::<SeedKey, _>::new(42_u32)));
  let cell = layer.build().expect("layer");
  assert_eq!(cell.value, 42);
  println!("026_layer_fn ok");
}
