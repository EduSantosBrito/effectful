//! Ex 108 — `Context` to `ServiceContext` interop.
//!
//! Shows how to build a compile-time `Context` with `ctx!`, convert it to a runtime
//! `ServiceContext`, then use typed service lookup via `Config::use_sync`.
use effectful::{
  ContextService, Effect, IntoServiceContext, MissingService, Service, ServiceContext,
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
  let static_ctx = effectful::ctx!(
    Config => Config { port: 8080 },
    Db => Db { id: 1 },
  );

  // Convert to runtime ServiceContext at the edge.
  let svc_ctx: ServiceContext = static_ctx.into_service_context();

  // Primary lookup path: typed service accessor.
  let program: Effect<u16, MissingService, ServiceContext> =
    Config::use_sync(|config| config.port);
  assert_eq!(run_blocking(program, svc_ctx.clone()), Ok(8080));

  // Runtime API still works for direct access.
  let config = svc_ctx.get_cloned::<Config>().expect("Config present");
  assert_eq!(config.port, 8080);

  let db = svc_ctx.get_cloned::<Db>().expect("Db present");
  assert_eq!(db.id, 1);

  println!("108_context_service_context_interop ok");
}
