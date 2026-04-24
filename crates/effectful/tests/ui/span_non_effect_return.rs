use effectful::span;

#[span]
fn work() -> u32 {
  1
}

fn main() {
  let _ = work();
}
