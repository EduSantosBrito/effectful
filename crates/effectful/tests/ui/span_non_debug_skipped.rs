use effectful::{Effect, Never, span, succeed};

struct NotDebug;

#[span(skip(value))]
fn work(value: NotDebug) -> Effect<(), Never, ()> {
  let _ = value;
  succeed::<(), Never, ()>(())
}

fn main() {
  let _ = work(NotDebug);
}
