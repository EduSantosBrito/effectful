//! Ex 022 — `Get::<K>::get` reads the head cell.
use effectful::{Get, Service, ctx};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct K;

fn main() {
  let env = ctx!(K => "here");
  assert_eq!(*Get::<K>::get(&env), "here");
  println!("022_get_here ok");
}
