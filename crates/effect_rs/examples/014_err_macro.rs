//! Ex 014 — `err!` builds nested `Or` error aliases.
use effect_rs::err;

type E = err!(u8 | u16);

fn main() {
  let left: Result<(), E> = Err(effect_rs::Or::Left(1_u8));
  let right: Result<(), E> = Err(effect_rs::Or::Right(2_u16));
  assert!(left.is_err());
  assert!(right.is_err());
  println!("014_err_macro ok");
}
