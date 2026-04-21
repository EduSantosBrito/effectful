//! Ex 002 — Failure surfaces at the program boundary as `Err`.
use effectful::{fail, run_blocking};

fn main() {
  let program = fail::<(), &'static str, ()>("not_ready");
  assert_eq!(run_blocking(program, ()), Err("not_ready"));
  println!("002_fail_boundary ok");
}
