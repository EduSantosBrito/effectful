//! Ex 023 — `get_path` follows `ThereHere` / `Skip` paths.
use effectful::{Service, ThereHere, ctx};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct FirstKey;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct SecondKey;

fn main() {
  let env = ctx!(FirstKey => 1_u8, SecondKey => 2_u16);
  assert_eq!(*env.get_path::<SecondKey, ThereHere>(), 2);
  println!("023_get_path ok");
}
