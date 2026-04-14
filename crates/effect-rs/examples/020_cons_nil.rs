//! Ex 020 — `Cons` / `Nil` heterogenous lists form environments.
use effect::{Cons, Nil, Tagged};

effect::service_key!(struct AKey);
effect::service_key!(struct BKey);

fn main() {
  let row = Cons(
    Tagged::<AKey, _>::new(1_u8),
    Cons(Tagged::<BKey, _>::new(2_u16), Nil),
  );
  assert_eq!(row.0.value, 1);
  assert_eq!(row.1.0.value, 2);
  println!("020_cons_nil ok");
}
