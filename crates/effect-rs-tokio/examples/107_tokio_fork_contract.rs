//! Ex 107 — `run_fork` on `TokioRuntime` runs the effect on the Tokio runtime; `join` returns the result.
//!
//! Run: `cargo run -p effect-tokio --example 107_tokio_fork_contract`

use effect::kernel::succeed;
use effect::run_fork;
use effect_tokio::TokioRuntime;

fn main() {
  let rt = TokioRuntime::new_current_thread().expect("tokio runtime should build");

  let forked = run_fork(&rt, || (succeed::<u8, (), ()>(5), ()));
  assert!(!forked.id().is_root());
  let out = rt.block_on(forked.join());
  assert_eq!(out, Ok(5));

  println!("107_tokio_fork_contract ok");
}
