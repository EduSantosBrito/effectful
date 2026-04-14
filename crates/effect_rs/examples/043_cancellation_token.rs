//! Ex 043 — `CancellationToken::cancel` flips the flag.
use effect_rs::CancellationToken;

fn main() {
  let t = CancellationToken::new();
  assert!(!t.is_cancelled());
  assert!(t.cancel());
  assert!(t.is_cancelled());
  println!("043_cancellation_token ok");
}
