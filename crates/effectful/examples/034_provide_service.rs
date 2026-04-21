//! Ex 034 — `provide_service` fixes the head tag using a value and shortens `R`.
use effectful::{
  Cons, Context, Get, Nil, ThereHere, ctx, effect, provide_service, run_blocking, service_key,
};

service_key!(struct GateKey);
service_key!(struct ValueKey);

type Full =
  Context<Cons<effectful::Service<GateKey, bool>, Cons<effectful::Service<ValueKey, i32>, Nil>>>;
type Short = Context<Cons<effectful::Service<ValueKey, i32>, Nil>>;

fn main() {
  let program = effect!(|r: &mut Full| {
    let on = bind* Ok::<_, ()>(*Get::<GateKey>::get(r));
    let v = bind* Ok::<_, ()>(*r.get_path::<ValueKey, ThereHere>());
    if on {
      v
    } else {
      0
    }
  });
  let short: Short = ctx!(ValueKey => 42);
  let peeled = provide_service(program, true);
  assert_eq!(run_blocking(peeled, short), Ok::<i32, ()>(42));
  println!("034_provide_service ok");
}
