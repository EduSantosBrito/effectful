//! Integration test: compile-time `Context` -> runtime `ServiceContext` interop.

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

mod context_to_service_context {
  use super::*;

  #[test]
  fn ctx_macro_interop() {
    let static_ctx = effectful::ctx!(Config => Config { port: 8080 });
    let env: ServiceContext = static_ctx.into_service_context();
    assert_eq!(env.get_cloned::<Config>(), Some(Config { port: 8080 }));
  }

  #[test]
  fn single_service_converts_and_looks_up() {
    let static_ctx = effectful::ctx!(Config => Config { port: 8080 });
    let svc_ctx: ServiceContext = static_ctx.into_service_context();
    assert_eq!(svc_ctx.get_cloned::<Config>(), Some(Config { port: 8080 }));
  }

  #[test]
  fn multiple_services_convert_and_lookup() {
    let static_ctx = effectful::ctx!(
      Config => Config { port: 3000 },
      Db => Db { id: 7 },
    );
    let svc_ctx: ServiceContext = static_ctx.into_service_context();
    assert_eq!(svc_ctx.get_cloned::<Config>(), Some(Config { port: 3000 }));
    assert_eq!(svc_ctx.get_cloned::<Db>(), Some(Db { id: 7 }));
  }

  #[test]
  fn effect_service_lookup_works_after_conversion() {
    let static_ctx = effectful::ctx!(Config => Config { port: 9090 });
    let svc_ctx: ServiceContext = static_ctx.into_service_context();

    let program: Effect<u16, MissingService, ServiceContext> =
      Config::use_sync(|config| config.port);

    assert_eq!(run_blocking(program, svc_ctx), Ok(9090));
  }

  #[test]
  fn ctx_macro_conversion_supports_service_use_sync_lookup() {
    let static_ctx = effectful::ctx!(
      Config => Config { port: 7000 },
      Db => Db { id: 3 },
    );
    let svc_ctx: ServiceContext = static_ctx.into_service_context();

    let program: Effect<u16, MissingService, ServiceContext> =
      Config::use_sync(|config| config.port);

    assert_eq!(run_blocking(program, svc_ctx), Ok(7000));
  }

  #[test]
  fn service_context_identity_noop() {
    let ctx = ServiceContext::empty().add(Config { port: 5000 });
    let converted = ctx.into_service_context();
    assert_eq!(
      converted.get_cloned::<Config>(),
      Some(Config { port: 5000 })
    );
  }
}
