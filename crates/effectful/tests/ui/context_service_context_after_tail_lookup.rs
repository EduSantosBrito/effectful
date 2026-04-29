use effectful::{Cons, Context, Nil, Tagged};
use effectful::{
  ContextService, Effect, IntoServiceContext, MissingService, Service, ServiceContext,
  run_blocking,
};

struct Header;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Config {
  port: u16,
}

fn main() {
  // Static Context with a non-service head and a self-keyed service tail.
  let ctx = Context::new(Cons(
    Tagged::<Header, _>::new(Header),
    Cons(Tagged::<Config, _>::new(Config { port: 8080 }), Nil),
  ));

  // Drop the head, keeping only the service tail.
  let tail_ctx = ctx.into_tail();

  // Convert the tail to a runtime ServiceContext.
  let env: ServiceContext = tail_ctx.into_service_context();

  // Lookup via the typed service helper compiles and runs.
  let program: Effect<u16, MissingService, ServiceContext> =
    Config::use_sync(|config| config.port);

  assert_eq!(run_blocking(program, env), Ok(8080));
}
