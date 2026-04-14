//! Ex 018 — `Exit` models completed runs (`succeed` / `fail`).
use effect_rs::{Exit, run_test, succeed};

fn main() {
  assert_eq!(
    run_test(succeed::<i32, (), ()>(42_i32), ()),
    Exit::succeed(42)
  );
  println!("018_exit_type ok");
}
