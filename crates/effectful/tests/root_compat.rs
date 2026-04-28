//! Root compatibility tests: verify that downstream crate API contracts are preserved.

use effectful::{
  Cons, Context, Effect, LayerBuild, Nil, Service, layer_service, layer_service_env,
  provide_service, service, service_env,
};

// ─── Fixtures ───────────────────────────────────────────────────────────────

effectful::service_key!(struct TestKey);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, effectful::Service)]
struct PrdService {
  value: u32,
}

// ─── Legacy Service<K, V> type alias ────────────────────────────────────────

#[test]
fn root_service_type_alias_constructible() {
  let _cell: Service<TestKey, i32> = Service::<TestKey, _>::new(42);
}

#[test]
fn root_service_type_alias_implements_get() {
  let ctx = Context::new(Cons(Service::<TestKey, _>::new(99), Nil));
  assert_eq!(*ctx.get::<TestKey>(), 99);
}

// ─── Legacy service constructors ────────────────────────────────────────────

#[test]
fn root_service_fn_builds_tagged_cell() {
  let cell = service::<TestKey, _>(77);
  assert_eq!(cell.value, 77);
}

#[test]
fn root_service_env_fn_builds_single_service_context() {
  let env = service_env::<TestKey, _>(55);
  assert_eq!(*env.get::<TestKey>(), 55);
}

// ─── Legacy layer constructors ──────────────────────────────────────────────

#[test]
fn root_layer_service_builds_cell() {
  let layer = layer_service::<TestKey, _>(123);
  let cell = layer.build().expect("layer builds");
  assert_eq!(cell.value, 123);
}

#[test]
fn root_layer_service_env_builds_context() {
  let layer = layer_service_env::<TestKey, _>(456);
  let env = layer.build().expect("layer builds");
  assert_eq!(*env.get::<TestKey>(), 456);
}

#[test]
fn root_provide_service_helper_provides_head() {
  let effect = Effect::new(|ctx: &mut Context<Cons<Service<TestKey, u8>, Nil>>| {
    Ok::<u8, ()>(*ctx.get::<TestKey>())
  });
  let provided = provide_service(effect, 42u8);
  let out = effectful::run_blocking(provided, Context::new(Nil));
  assert_eq!(out, Ok(42));
}

// ─── PRD trait still available via ContextService alias ─────────────────────

#[test]
fn context_service_trait_methods_still_work() {
  let ctx = PrdService { value: 7 }.to_context();
  assert_eq!(ctx.get::<PrdService>().map(|s| s.value), Some(7));
}

// ─── LayerBuild explicit trait call ─────────────────────────────────────────

#[test]
fn layer_build_trait_method_works_via_explicit_call() {
  let layer = effectful::layer::LayerFn(|| Ok::<i32, ()>(42));
  let out = LayerBuild::build(&layer).expect("build");
  assert_eq!(out, 42);
}

// ─── PRD Layer via Service::layer() ─────────────────────────────────────────

#[test]
fn service_layer_returns_prd_layer() {
  let svc = PrdService { value: 99 };
  let layer = svc.layer();
  let ctx = effectful::run_blocking(layer.build(), ()).expect("build");
  assert_eq!(ctx.get::<PrdService>().map(|s| s.value), Some(99));
}

// ─── Downstream-style imports: effectful_logger ─────────────────────────────

#[test]
fn downstream_logger_imports_available_at_root() {
  // effectful_logger uses these at root
  let _: effectful::EffectHashMap<String, String> = effectful::collections::hash_map::empty();
  let _: effectful::FiberRef<i32> =
    effectful::run_blocking(effectful::FiberRef::make(|| 0), ()).unwrap();
  let _: Option<effectful::Never> = None;
}

// ─── Downstream-style imports: effectful_reqwest ────────────────────────────

#[test]
fn downstream_reqwest_imports_available_at_root() {
  // effectful_reqwest uses these at root
  let _: effectful::Pool<i32, effectful::Never> =
    effectful::run_blocking(effectful::Pool::make(1, || effectful::succeed(0)), ()).unwrap();
  let _ = effectful::Scope::make();
}

// ─── Derive macro still works at root ───────────────────────────────────────

#[test]
fn derive_service_macro_still_works() {
  #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, effectful::Service)]
  struct DerivedSvc {
    n: i8,
  }
  let ctx = DerivedSvc { n: 3 }.to_context();
  assert_eq!(ctx.get::<DerivedSvc>().map(|s| s.n), Some(3));
}
