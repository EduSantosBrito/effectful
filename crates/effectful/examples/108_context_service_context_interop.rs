//! Ex 108 — `Context` to `ServiceContext` interop.
//!
//! Shows how to build a compile-time `Context`, convert it to a runtime
//! `ServiceContext`, then use service lookup via `Config::use_sync` / `Effect::service`.
use effectful::{
  Context, ContextService, Effect, IntoServiceContext, MissingService, Service, ServiceContext,
  run_blocking,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Config {
  port: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Db {
  id: u8,
}

fn main() {
  // Build a compile-time Context with self-keyed service cells.
  let ctx = Context::new(effectful::Cons(
    effectful::Tagged::<Config, _>::new(Config { port: 8080 }),
    effectful::Cons(
      effectful::Tagged::<Db, _>::new(Db { id: 1 }),
      effectful::Nil,
    ),
  ));

  // Convert to runtime ServiceContext at the edge.
  let svc_ctx: ServiceContext = ctx.into_service_context();

  // Service lookup still works via the normal runtime API.
  let config = svc_ctx.get_cloned::<Config>().expect("Config present");
  assert_eq!(config.port, 8080);

  let db = svc_ctx.get_cloned::<Db>().expect("Db present");
  assert_eq!(db.id, 1);

  // Effects that require ServiceContext also work.
  let program: Effect<u16, MissingService, ServiceContext> = Config::use_sync(|config| config.port);

  assert_eq!(run_blocking(program, svc_ctx), Ok(8080));
  println!("108_context_service_context_interop ok");
}
