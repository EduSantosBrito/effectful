use effectful::{Effect, effect, run_blocking, succeed};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    bind* succeed::<i32, (), ()>(42)
  };

  assert_eq!(run_blocking(program, ()), Ok(42));
}
