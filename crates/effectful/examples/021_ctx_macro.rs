//! Ex 021 — `ctx!` builds a `Context` from key/value pairs.
use effectful::{Cons, Context, Get, Nil, Service, Tagged, ctx, run_blocking, succeed};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct PortKey;

type Env = Context<Cons<Tagged<PortKey, u16>, Nil>>;

fn main() {
  let env: Env = ctx!(PortKey => 8080_u16);
  assert_eq!(*Get::<PortKey>::get(&env), 8080);
  assert_eq!(
    run_blocking(succeed::<(), (), Env>(()), env),
    Ok::<(), ()>(())
  );
  println!("021_ctx_macro ok");
}
