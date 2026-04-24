//! Ex 107 — Effectful spans can also emit Rust `tracing` spans/events.
use effectful::{
  Never, TracingConfig, emit_current_span_event, install_tracing_layer, run_blocking, span, succeed,
};

#[span(name = "bridge.demo", fields(component = "example"))]
fn bridged_work() -> effectful::Effect<(), Never, ()> {
  emit_current_span_event::<()>("bridge.event").flat_map(|_| succeed::<(), Never, ()>(()))
}

fn main() {
  let _ = tracing_subscriber::fmt().without_time().try_init();
  let _ = run_blocking(
    install_tracing_layer(TracingConfig::enabled_with_tracing_bridge()),
    (),
  );
  assert_eq!(run_blocking(bridged_work(), ()), Ok(()));
  println!("107_tracing_bridge ok");
}
