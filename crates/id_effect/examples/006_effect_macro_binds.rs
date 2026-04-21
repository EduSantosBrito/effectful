//! Ex 006 — `effect!` binds with `let x = bind* expr`.
use id_effect::{Effect, effect, run_blocking, succeed};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    let x = bind* succeed(6_i32);
    let y = bind* succeed(7_i32);
    x * y
  };
  assert_eq!(run_blocking(program, ()), Ok(42));
  println!("006_effect_macro_binds ok");
}
