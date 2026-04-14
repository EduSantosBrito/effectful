//! Ex 060 — `Stream::range` materializes as a single chunk, then ends.
use effect_rs::{Stream, run_blocking};

fn main() {
  let s = Stream::range(1, 5).run_collect();
  assert_eq!(run_blocking(s, ()), Ok(vec![1, 2, 3, 4]));
  println!("060_stream_range ok");
}
