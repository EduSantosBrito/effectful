# Observability

Effectful includes lazy span instrumentation for production tracing without forcing an OpenTelemetry SDK or exporter dependency into core.

Use spans when you want to know when an `Effect` starts, succeeds, fails, how long it ran, what trace/span identity it had, and how it relates to parent spans.

## Manual Spans

Wrap any existing effect with `with_span` or the method form.

```rust
use effectful::{TracingConfig, install_tracing_layer, run_blocking, succeed};

let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());

let effect = succeed::<u32, (), ()>(42).with_span("manual.demo");
assert_eq!(run_blocking(effect, ()), Ok(42));
```

Use `SpanOptions` for levels and typed startup attributes.

```rust
use effectful::{SpanLevel, SpanOptions, succeed};

let effect = succeed::<(), (), ()>(()).with_span_options(
  SpanOptions::new("manual.options")
    .with_level(SpanLevel::Debug)
    .with_attribute("cached", true)
    .with_attribute("attempt", 2_i32),
);
```

## Function Spans

Use `#[effectful::span]` on functions returning `Effect`.

```rust
use effectful::{Effect, Never, span, succeed};

#[span(name = "user.load", level = debug)]
fn load_user(id: u32) -> Effect<u32, Never, ()> {
  succeed::<u32, Never, ()>(id + 1)
}
```

Calling `load_user` only constructs an effect. The span starts when that returned effect is executed.

By default, span names are `module_path::function_name`, and function arguments are captured with `Debug` formatting as string attributes.

## Privacy Controls

Skip sensitive, large, or non-`Debug` arguments with `skip(...)`.

```rust
use effectful::{Effect, Never, span, succeed};

struct Secret(String);

#[span(name = "auth.login", skip(password))]
fn login(user_id: u64, password: Secret) -> Effect<(), Never, ()> {
  let _ = password.0.len();
  succeed::<(), Never, ()>(())
}
```

Use `skip_all` to capture only explicit fields.

```rust
#[effectful::span(skip_all, fields(route = "/users/:id", cache_hit = true))]
fn route() -> effectful::Effect<(), effectful::Never, ()> {
  effectful::succeed::<(), effectful::Never, ()>(())
}
```

Field values can be typed, display-formatted with `%`, or debug-formatted with `?`.

```rust
#[effectful::span(fields(count = 3_i32, user = %user_id, payload = ?payload))]
fn work(user_id: u64, payload: Vec<u8>) -> effectful::Effect<(), effectful::Never, ()> {
  effectful::succeed::<(), effectful::Never, ()>(())
}
```

Typed fields preserve strings, booleans, signed integers, and floats. `%` and `?` fields are stored as strings.

## Disabled Tracing

When tracing is disabled or not installed, span hooks are no-ops. For non-async `#[span]` functions, the span macro also avoids formatting captured arguments while tracing is disabled or the span is sampled out.

```rust
use effectful::{TracingConfig, install_tracing_layer, run_blocking};

let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
```

## Trace Context

Every span record has local OpenTelemetry-shaped identity: `TraceId`, `SpanId`, `TraceFlags`, and parent span id.

Use W3C `traceparent` helpers at integration boundaries.

```rust
use effectful::SpanContext;

let context = SpanContext::from_traceparent(
  "00-01010101010101010101010101010101-0202020202020202-01",
)?;

let header = context.to_traceparent();
# Ok::<(), effectful::TraceParentParseError>(())
```

## Rust tracing Bridge

Enable the Rust `tracing` bridge when you want existing subscribers, Loki pipelines, or `tracing-opentelemetry` stacks to consume effectful spans. Use `TracingConfig::tracing_bridge_only()` for production forwarding without retaining `snapshot_tracing()` buffers; use `enabled_with_tracing_bridge()` when you also need in-memory snapshots.

```rust
use effectful::{TracingConfig, install_tracing_layer, run_blocking};

let _ = tracing_subscriber::fmt().try_init();
let _ = run_blocking(
  install_tracing_layer(TracingConfig::tracing_bridge_only()),
  (),
);
```

Full snapshot+bridge mode emits spans named `effectful.span` with `otel_name`, `otel_trace_id`, `otel_span_id`, `otel_parent_span_id`, `otel_trace_flags`, status, duration, and attribute fields. Span events emitted with `emit_current_span_event` are emitted as Rust `tracing` events inside the current span.

Bridge-only mode emits name-only Rust `tracing` spans with `otel_name`. It intentionally skips trace id generation, parent lookup, attribute recording, and snapshot storage so it can stay production-oriented.

For high-throughput services, use an async/batched subscriber/exporter. Subscriber formatting, locks, and exporter backpressure usually dominate span overhead.

## Sampling

Use sampling on hot paths. `default_sample_rate` applies globally; `SpanOptions::sample_rate` and `#[effectful::span(sample = ...)]` override it per span. Rates are rounded down to a power-of-two cadence, so `0.3` records at most one in four spans.

```rust
use effectful::{TracingConfig, install_tracing_layer, run_blocking};

let mut config = TracingConfig::tracing_bridge_only();
config.default_sample_rate = 0.0625;
let _ = run_blocking(install_tracing_layer(config), ());
```

Snapshot mode is best for tests, local debugging, and diagnostics. Bridge-only plus sampling is the production-oriented path for very small effects.

See examples `106_span_macro.rs` and `107_tracing_bridge.rs` for executable coverage.
