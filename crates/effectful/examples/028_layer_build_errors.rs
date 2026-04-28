//! Ex 028 — Failed layers propagate `Err` from `build`.
use effectful::{LayerBuild, LayerFn, Service, Tagged};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct K;

fn main() {
  let bad = LayerFn(|| Err::<Tagged<K, u8>, &'static str>("no"));
  assert_eq!(bad.build(), Err("no"));
  println!("028_layer_build_errors ok");
}
