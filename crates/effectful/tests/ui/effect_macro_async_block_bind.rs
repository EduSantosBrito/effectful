use effectful::{Effect, effect, from_async, run_blocking};

fn main() {
  let program: Effect<i32, (), ()> = effect! {
    bind* from_async(|_| async move { Ok::<i32, ()>(42) })
  };

  assert_eq!(run_blocking(program, ()), Ok(42));
}
