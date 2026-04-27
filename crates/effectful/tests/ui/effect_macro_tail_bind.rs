use effectful::{Effect, effect, run_blocking, succeed};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    bind* succeed(42_i32)
  };

  assert_eq!(run_blocking(program, ()), Ok(42));
}
