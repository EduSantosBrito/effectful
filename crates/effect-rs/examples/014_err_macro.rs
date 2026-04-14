//! Ex 014 — `err!` builds nested `Or` error aliases.
use effect::err;

type E = err!(u8 | u16);

fn main() {
  let left: Result<(), E> = Err(effect::Or::Left(1_u8));
  let right: Result<(), E> = Err(effect::Or::Right(2_u16));
  assert!(left.is_err());
  assert!(right.is_err());
  println!("014_err_macro ok");
}
