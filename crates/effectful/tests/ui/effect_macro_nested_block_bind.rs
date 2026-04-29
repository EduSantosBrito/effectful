use effectful::{Effect, effect, run_blocking, succeed};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    let x = { bind* succeed(20_i32) };
    x + 22
  };

  assert_eq!(run_blocking(program, ()), Ok(42));
}
