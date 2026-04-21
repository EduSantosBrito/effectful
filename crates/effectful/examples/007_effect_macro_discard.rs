//! Ex 007 — Discard a unit effect with `bind* expr;`.
use effectful::{Effect, effect, run_blocking, succeed};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    bind* succeed(());
    bind* succeed(());
    42_i32
  };
  assert_eq!(run_blocking(program, ()), Ok(42));
  println!("007_effect_macro_discard ok");
}
