use effectful::{Effect, Never, span, succeed};

struct NotDebug;

#[span]
fn work(value: NotDebug) -> Effect<(), Never, ()> {
  let _ = value;
  succeed::<(), Never, ()>(())
}

fn main() {
  let _ = work(NotDebug);
}
