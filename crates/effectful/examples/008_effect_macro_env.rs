//! Ex 008 — `effect!` closure receives `&mut R` (the environment).
use effectful::{Cons, Context, Get, Nil, Service, Tagged, ctx, effect, run_blocking, succeed};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct CounterKey;

type Env = Context<Cons<Tagged<CounterKey, i32>, Nil>>;

fn main() {
  let program = effect!(|r: &mut Env| {
    let n = bind * succeed(*Get::<CounterKey>::get(r));
    n + 1
  });
  let env = ctx!(CounterKey => 41_i32);
  assert_eq!(run_blocking(program, env), Ok::<i32, ()>(42));
  println!("008_effect_macro_env ok");
}
