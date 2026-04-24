//! Ex 106 — `#[effectful::span]` instruments functions returning `Effect` lazily.
use effectful::{
  Effect, Never, SpanAttributeValue, SpanLevel, TracingConfig, install_tracing_layer, run_blocking,
  snapshot_tracing, span, succeed,
};
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

struct Secret(String);

struct CountedDebug {
  hits: Arc<AtomicUsize>,
}

impl fmt::Debug for CountedDebug {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.hits.fetch_add(1, Ordering::SeqCst);
    f.write_str("CountedDebug")
  }
}

#[span(
  name = "user.load",
  level = debug,
  skip(secret),
  fields(cache_hit = true, id_display = %id)
)]
fn load_user(id: u32, secret: Secret) -> Effect<u32, Never, ()> {
  let _ = secret.0.len();
  succeed::<u32, Never, ()>(id + 1)
}

#[span(name = "user.async_load", skip_all)]
async fn async_load_user(id: u32) -> Effect<u32, Never, ()> {
  succeed::<u32, Never, ()>(id + 2)
}

#[span(name = "disabled.capture")]
fn disabled_capture(value: CountedDebug) -> Effect<(), Never, ()> {
  Effect::new(move |_env: &mut ()| {
    let _ = value.hits.load(Ordering::SeqCst);
    Ok(())
  })
}

#[span(name = "sampled.out", sample = 0.0)]
fn sampled_out_capture(value: CountedDebug) -> Effect<(), Never, ()> {
  Effect::new(move |_env: &mut ()| {
    let _ = value.hits.load(Ordering::SeqCst);
    Ok(())
  })
}

fn main() {
  let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
  let eff = load_user(41, Secret("redacted".to_string()));
  assert!(snapshot_tracing().spans.is_empty());

  assert_eq!(run_blocking(eff, ()), Ok(42));
  let snap = snapshot_tracing();
  let span = snap
    .spans
    .iter()
    .find(|span| span.name == "user.load")
    .expect("span recorded");
  assert_eq!(span.level, SpanLevel::Debug);
  assert_eq!(
    span.attributes.get("id"),
    Some(&SpanAttributeValue::String("41".to_string()))
  );
  assert_eq!(span.attributes.get("secret"), None);
  assert_eq!(
    span.attributes.get("cache_hit"),
    Some(&SpanAttributeValue::Bool(true))
  );
  assert_eq!(
    span.attributes.get("id_display"),
    Some(&SpanAttributeValue::String("41".to_string()))
  );

  let eff = pollster::block_on(async_load_user(40));
  assert_eq!(run_blocking(eff, ()), Ok(42));
  assert!(
    snapshot_tracing()
      .spans
      .iter()
      .any(|span| span.name == "user.async_load")
  );

  let hits = Arc::new(AtomicUsize::new(0));
  let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
  let eff = disabled_capture(CountedDebug {
    hits: Arc::clone(&hits),
  });
  assert_eq!(run_blocking(eff, ()), Ok(()));
  assert_eq!(hits.load(Ordering::SeqCst), 0);

  let hits = Arc::new(AtomicUsize::new(0));
  let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
  let eff = sampled_out_capture(CountedDebug {
    hits: Arc::clone(&hits),
  });
  assert_eq!(run_blocking(eff, ()), Ok(()));
  assert_eq!(hits.load(Ordering::SeqCst), 0);
  assert!(snapshot_tracing().spans.is_empty());
  println!("106_span_macro ok");
}
