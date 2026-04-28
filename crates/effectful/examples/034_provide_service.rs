//! Ex 034 — `Effect::provide` supplies services through a `Layer`.
use effectful::{
  ContextService, Effect, Layer, MissingService, Service, ServiceContext, run_blocking,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Gate {
  on: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Value {
  n: i32,
}

fn main() {
  let program: Effect<i32, MissingService, ServiceContext> =
    Gate::use_(|gate| Value::use_sync(move |value| if gate.on { value.n } else { 0 }));
  let gate = Layer::<Gate, MissingService>::succeed(Gate { on: true });
  let value = Layer::<Value, MissingService>::succeed(Value { n: 42 });

  assert_eq!(run_blocking(program.provide(gate.merge(value)), ()), Ok(42));
  println!("034_provide_service ok");
}
