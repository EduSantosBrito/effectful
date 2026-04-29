//! Lightweight tracing hooks for effect/fiber observability.

use super::traceparent::{SpanContext, SpanId, TraceFlags, TraceId};
use crate::Effect;
use crate::collections::EffectHashMap;
use crate::collections::hash_map;
use crate::concurrency::fiber_ref::FiberRef;
use crate::effect;
use crate::kernel::box_future;
use crate::kernel::effect::{ProgramOp, SyncStep, start_async_operation, start_effect};
use crate::runtime::{Never, run_blocking};
use crate::scheduling::Clock;
use std::borrow::Cow;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

mod annotate_current_span_seal {
  pub(super) trait Success {}
  pub(super) trait Error {}
}

/// Success type for [`annotate_current_span`]. Implemented for [`unit`](()) only (sealed) so `A`
/// infers at call sites while the API stays `Effect<A, E, R>`-shaped.
#[allow(private_bounds)] // seal traits in `annotate_current_span_seal` are intentionally private
pub trait AnnotateCurrentSpanSuccess: From<()> + annotate_current_span_seal::Success {}

/// Error type for [`annotate_current_span`]. Implemented for [`Never`] only (sealed); the helper is
/// infallible.
#[allow(private_bounds)]
pub trait AnnotateCurrentSpanErr: From<Never> + annotate_current_span_seal::Error {}

impl annotate_current_span_seal::Success for () {}
impl AnnotateCurrentSpanSuccess for () {}

impl annotate_current_span_seal::Error for Never {}
impl AnnotateCurrentSpanErr for Never {}

/// Global tracing toggle installed by [`install_tracing_layer`].
#[derive(Clone, Debug, PartialEq)]
pub struct TracingConfig {
  /// When `false`, hooks and span recording are no-ops.
  pub enabled: bool,
  /// When `true`, effectful spans are also emitted through Rust `tracing`.
  pub bridge_to_tracing: bool,
  /// When `true`, spans/events are copied into [`snapshot_tracing`] buffers.
  pub record_in_memory: bool,
  /// Fraction of spans to instrument (1.0 = always). Rounded down to a power-of-two cadence.
  pub default_sample_rate: f64,
}

impl Default for TracingConfig {
  fn default() -> Self {
    Self {
      enabled: false,
      bridge_to_tracing: false,
      record_in_memory: false,
      default_sample_rate: 1.0,
    }
  }
}

impl TracingConfig {
  /// Config with tracing turned on.
  #[inline]
  pub fn enabled() -> Self {
    Self {
      enabled: true,
      bridge_to_tracing: false,
      record_in_memory: true,
      default_sample_rate: 1.0,
    }
  }

  /// Config with in-memory tracing and the Rust `tracing` bridge turned on.
  #[inline]
  pub fn enabled_with_tracing_bridge() -> Self {
    Self {
      enabled: true,
      bridge_to_tracing: true,
      record_in_memory: true,
      default_sample_rate: 1.0,
    }
  }

  /// Config for forwarding spans to Rust `tracing` without retaining snapshots.
  #[inline]
  pub fn tracing_bridge_only() -> Self {
    Self {
      enabled: true,
      bridge_to_tracing: true,
      record_in_memory: false,
      default_sample_rate: 1.0,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub(crate) enum TracingMode {
  /// Tracing is disabled — all span work is a no-op.
  Disabled = 0,
  /// Bridge-only: forward to `tracing`, no in-memory recording, no FiberRef stack.
  BridgeOnly = 1,
  /// Bridge-only but no subscriber present — fully no-op.
  BridgeOnlyNoSubscriber = 2,
  /// Full snapshot recording (in-memory). Optionally also bridges to `tracing`.
  Snapshot = 3,
  /// Full snapshot with tracing bridge.
  SnapshotBridge = 4,
}

impl TracingMode {
  const fn from_config(config: &TracingConfig, subscriber_available: bool) -> Self {
    if !config.enabled {
      return Self::Disabled;
    }
    if !config.record_in_memory {
      if !config.bridge_to_tracing || !subscriber_available {
        return Self::Disabled;
      }
      return Self::BridgeOnly;
    }
    if config.bridge_to_tracing {
      Self::SnapshotBridge
    } else {
      Self::Snapshot
    }
  }

  #[inline]
  const fn is_disabled(self) -> bool {
    matches!(self, Self::Disabled)
  }

  #[inline]
  const fn is_bridge_only(self) -> bool {
    matches!(self, Self::BridgeOnly)
  }
}

/// Provides trace/span identity for new spans.
pub trait TraceContextProvider: Send + Sync + 'static {
  /// Context for a new root span.
  fn root_context(&self) -> SpanContext;
  /// Context for a child span of `parent`.
  fn child_context(&self, parent: &SpanContext) -> SpanContext;
}

static GLOBAL_SEQ_SPAN_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Deterministic monotonic id provider used by default and by tests.
#[derive(Debug, Default)]
pub struct SequentialTraceContextProvider {
  next_trace: AtomicU64,
}

impl SequentialTraceContextProvider {
  /// Creates a provider whose first generated trace counter is `1`.
  #[inline]
  pub fn new() -> Self {
    Self::default()
  }

  fn next_trace_id(&self) -> TraceId {
    TraceId::from_bytes(counter_to_trace_bytes(
      self.next_trace.fetch_add(1, Ordering::Relaxed) + 1,
    ))
  }

  fn next_span_id(&self) -> SpanId {
    SpanId::from_bytes((GLOBAL_SEQ_SPAN_COUNTER.fetch_add(1, Ordering::Relaxed) + 1).to_be_bytes())
  }
}

impl TraceContextProvider for SequentialTraceContextProvider {
  fn root_context(&self) -> SpanContext {
    SpanContext {
      trace_id: self.next_trace_id(),
      span_id: self.next_span_id(),
      trace_flags: TraceFlags::DEFAULT,
    }
  }

  fn child_context(&self, parent: &SpanContext) -> SpanContext {
    SpanContext {
      trace_id: parent.trace_id,
      span_id: self.next_span_id(),
      trace_flags: parent.trace_flags,
    }
  }
}

/// Span severity level; maps directly to common `tracing` filters later.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SpanLevel {
  /// Trace-level span.
  Trace,
  /// Debug-level span.
  Debug,
  /// Info-level span.
  #[default]
  Info,
  /// Warning-level span.
  Warn,
  /// Error-level span.
  Error,
}

/// OTel-shaped span status.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SpanStatus {
  /// Span has not completed yet or did not set an explicit status.
  #[default]
  Unset,
  /// Span completed successfully.
  Ok,
  /// Span completed with an error. Error values are intentionally not captured.
  Error,
}

/// Typed span attribute value. Formatted Debug/Display fields should enter as `String`.
#[derive(Clone, Debug, PartialEq)]
pub enum SpanAttributeValue {
  /// UTF-8 string attribute.
  String(String),
  /// Boolean attribute.
  Bool(bool),
  /// Signed integer attribute.
  I64(i64),
  /// Floating-point attribute.
  F64(f64),
}

impl From<String> for SpanAttributeValue {
  fn from(value: String) -> Self {
    Self::String(value)
  }
}

impl From<&str> for SpanAttributeValue {
  fn from(value: &str) -> Self {
    Self::String(value.to_owned())
  }
}

impl From<bool> for SpanAttributeValue {
  fn from(value: bool) -> Self {
    Self::Bool(value)
  }
}

impl From<i64> for SpanAttributeValue {
  fn from(value: i64) -> Self {
    Self::I64(value)
  }
}

impl From<i32> for SpanAttributeValue {
  fn from(value: i32) -> Self {
    Self::I64(i64::from(value))
  }
}

impl From<f64> for SpanAttributeValue {
  fn from(value: f64) -> Self {
    Self::F64(value)
  }
}

/// Domain event recorded inside a span.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanEvent {
  /// Event name.
  pub name: String,
  /// Event attributes.
  pub attributes: EffectHashMap<String, SpanAttributeValue>,
  /// Monotonic timestamp captured when the event was recorded.
  pub occurred_at: Instant,
}

/// Options used when starting a span.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanOptions {
  /// Span name.
  pub name: Cow<'static, str>,
  /// Span level. Defaults to [`SpanLevel::Info`].
  pub level: SpanLevel,
  /// Initial span attributes captured when the effect starts.
  pub attributes: EffectHashMap<String, SpanAttributeValue>,
  /// Per-span sample rate override. `None` inherits [`TracingConfig::default_sample_rate`].
  pub sample_rate: Option<f64>,
}

impl SpanOptions {
  /// Creates options for an INFO span with no initial attributes.
  #[inline]
  pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
    Self {
      name: name.into(),
      level: SpanLevel::Info,
      attributes: EffectHashMap::new(),
      sample_rate: None,
    }
  }

  /// Sets the span level.
  #[inline]
  pub fn with_level(mut self, level: SpanLevel) -> Self {
    self.level = level;
    self
  }

  /// Adds an initial typed attribute.
  #[inline]
  pub fn with_attribute(
    mut self,
    key: impl Into<String>,
    value: impl Into<SpanAttributeValue>,
  ) -> Self {
    self.attributes = hash_map::set(&self.attributes, key.into(), value.into());
    self
  }
}

/// Lifecycle markers for effects wrapped in [`with_span`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EffectEvent {
  /// Entered a named span.
  Start {
    /// Span name (matches [`with_span`] argument).
    span: String,
  },
  /// Effect under the span completed successfully.
  Success {
    /// Span name (matches [`with_span`] argument).
    span: String,
  },
  /// Effect under the span failed.
  Failure {
    /// Span name (matches [`with_span`] argument).
    span: String,
  },
}

/// Coarse fiber lifecycle signals (opt-in via [`emit_fiber_event`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FiberEvent {
  /// Fiber started work.
  Spawn {
    /// Opaque fiber identifier string.
    fiber_id: String,
  },
  /// Fiber finished normally.
  Complete {
    /// Opaque fiber identifier string.
    fiber_id: String,
  },
  /// Fiber was interrupted.
  Interrupt {
    /// Opaque fiber identifier string.
    fiber_id: String,
  },
}

/// Recorded span instance with OTel-shaped identity, parent, timing, status, attributes, and events.
#[derive(Clone, Debug, PartialEq)]
pub struct SpanRecord {
  /// Span name.
  pub name: String,
  /// Span context.
  pub context: SpanContext,
  /// Parent span id, when this span is nested under another effectful span.
  pub parent_span_id: Option<SpanId>,
  /// Span level.
  pub level: SpanLevel,
  /// Final span status.
  pub status: SpanStatus,
  /// Typed attributes merged from initial options and current-span annotations.
  pub attributes: EffectHashMap<String, SpanAttributeValue>,
  /// Span events.
  pub events: Vec<SpanEvent>,
  /// Monotonic start timestamp.
  pub started_at: Instant,
  /// Monotonic end timestamp, present after completion.
  pub ended_at: Option<Instant>,
}

impl SpanRecord {
  /// Returns the completed duration when the span has ended.
  #[inline]
  pub fn duration(&self) -> Option<Duration> {
    self
      .ended_at
      .map(|ended_at| ended_at.duration_since(self.started_at))
  }
}

/// One frame on the fiber-local span stack (`TracingFiberRefs::span_stack`).
#[derive(Clone, Debug)]
pub struct LogSpan {
  /// Name of the active span frame.
  pub name: Cow<'static, str>,
  /// Context of the active span frame.
  pub context: SpanContext,
  /// Rust `tracing` span used by the optional bridge.
  pub tracing_span: Option<::tracing::Span>,
}

/// Point-in-time copy of recorded tracing buffers.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TracingSnapshot {
  /// Ordered [`EffectEvent`] stream since last install.
  pub effect_events: Vec<EffectEvent>,
  /// Ordered [`FiberEvent`] stream since last install.
  pub fiber_events: Vec<FiberEvent>,
  /// Span records with merged annotations.
  pub spans: Vec<SpanRecord>,
}

/// Receives tracing lifecycle records.
pub trait TraceCollector: Send + Sync + 'static {
  /// Records a span start.
  fn span_started(&self, span: SpanRecord);
  /// Records a span completion.
  fn span_ended(
    &self,
    span_id: SpanId,
    attributes: EffectHashMap<String, SpanAttributeValue>,
    status: SpanStatus,
    ended_at: Instant,
  );
  /// Records an event emitted on a span.
  fn span_event(
    &self,
    span_id: SpanId,
    name: String,
    attributes: EffectHashMap<String, SpanAttributeValue>,
    occurred_at: Instant,
  );
  /// Records an effect lifecycle event.
  fn effect_event(&self, event: EffectEvent);
  /// Records a fiber lifecycle event.
  fn fiber_event(&self, event: FiberEvent);
}

/// Default collector adapter backed by the installed global tracing state.
#[derive(Debug, Default)]
pub struct GlobalTraceCollector;

impl TraceCollector for GlobalTraceCollector {
  fn span_started(&self, span: SpanRecord) {
    with_state_mut(|state| {
      state.spans.push(span);
    });
  }

  fn span_ended(
    &self,
    span_id: SpanId,
    attributes: EffectHashMap<String, SpanAttributeValue>,
    status: SpanStatus,
    ended_at: Instant,
  ) {
    with_state_mut(|state| {
      if let Some(span) = state
        .spans
        .iter_mut()
        .rev()
        .find(|span| span.context.span_id == span_id)
      {
        span.attributes = attributes;
        span.status = status;
        span.ended_at = Some(ended_at);
      }
    });
  }

  fn span_event(
    &self,
    span_id: SpanId,
    name: String,
    attributes: EffectHashMap<String, SpanAttributeValue>,
    occurred_at: Instant,
  ) {
    with_state_mut(|state| {
      if let Some(span) = state
        .spans
        .iter_mut()
        .rev()
        .find(|span| span.context.span_id == span_id)
      {
        span.events.push(SpanEvent {
          name,
          attributes,
          occurred_at,
        });
      }
    });
  }

  fn effect_event(&self, event: EffectEvent) {
    with_state_mut(|state| {
      state.effect_events.push(event);
    });
  }

  fn fiber_event(&self, event: FiberEvent) {
    with_state_mut(|state| {
      state.fiber_events.push(event);
    });
  }
}

struct TraceState {
  config: TracingConfig,
  effect_events: Vec<EffectEvent>,
  fiber_events: Vec<FiberEvent>,
  spans: Vec<SpanRecord>,
  context_provider: Arc<dyn TraceContextProvider>,
  clock_now: Arc<dyn Fn() -> Instant + Send + Sync>,
}

impl Default for TraceState {
  fn default() -> Self {
    Self {
      config: TracingConfig::default(),
      effect_events: Vec::new(),
      fiber_events: Vec::new(),
      spans: Vec::new(),
      context_provider: Arc::new(SequentialTraceContextProvider::new()),
      clock_now: Arc::new(Instant::now),
    }
  }
}

static TRACE_STATE: OnceLock<Mutex<TraceState>> = OnceLock::new();
static TRACING_MODE: AtomicU64 = AtomicU64::new(TracingMode::Disabled as u64);
static SAMPLING_MASK: AtomicU64 = AtomicU64::new(0);
const SAMPLING_NEVER: u64 = u64::MAX;

std::thread_local! {
  static LOCAL_SPAN_COUNTER: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// Fiber-local span stack and current-span annotation map (see beads n48f).
#[derive(Clone)]
pub struct TracingFiberRefs {
  /// Current span stack for this fiber.
  pub span_stack: FiberRef<Vec<LogSpan>>,
  /// Mutable attributes for the innermost span.
  pub span_annotations: FiberRef<EffectHashMap<String, SpanAttributeValue>>,
}

struct SpanStackPopGuard {
  refs: TracingFiberRefs,
  active: bool,
}

impl SpanStackPopGuard {
  fn new(refs: TracingFiberRefs) -> Self {
    Self { refs, active: true }
  }

  fn pop(&mut self) {
    if !self.active {
      return;
    }
    run_blocking(
      self.refs.span_stack.update(|mut v| {
        v.pop();
        v
      }),
      (),
    )
    .expect("span_stack pop");
    self.active = false;
  }
}

impl Drop for SpanStackPopGuard {
  fn drop(&mut self) {
    if !self.active {
      return;
    }
    let _ = run_blocking(
      self.refs.span_stack.update(|mut v| {
        v.pop();
        v
      }),
      (),
    );
    self.active = false;
  }
}

static TRACING_FIBER_REFS: OnceLock<TracingFiberRefs> = OnceLock::new();

fn trace_state() -> &'static Mutex<TraceState> {
  TRACE_STATE.get_or_init(|| Mutex::new(TraceState::default()))
}

#[inline]
fn tracing_mode() -> TracingMode {
  let raw = TRACING_MODE.load(Ordering::Relaxed);
  match raw {
    0 => TracingMode::Disabled,
    1 => TracingMode::BridgeOnly,
    2 => TracingMode::BridgeOnlyNoSubscriber,
    3 => TracingMode::Snapshot,
    4 => TracingMode::SnapshotBridge,
    _ => TracingMode::Disabled,
  }
}

#[inline]
fn should_sample(sample_rate_override: Option<f64>) -> bool {
  let mask = if let Some(rate) = sample_rate_override {
    sampling_mask(rate)
  } else {
    SAMPLING_MASK.load(Ordering::Relaxed)
  };
  if mask == 0 {
    return true;
  }
  if mask == SAMPLING_NEVER {
    return false;
  }
  LOCAL_SPAN_COUNTER.with(|cell| {
    let counter = cell.get();
    cell.set(counter.wrapping_add(1));
    (counter & mask) == 0
  })
}

/// Returns true when the in-memory tracing layer is currently enabled.
#[doc(hidden)]
pub fn tracing_enabled() -> bool {
  !tracing_mode().is_disabled()
}

fn current_tracing_subscriber_available() -> bool {
  ::tracing::dispatcher::get_default(|dispatch| {
    !dispatch.is::<::tracing::subscriber::NoSubscriber>()
  })
}

/// Computes a power-of-two sampling mask from the given rate.
/// Rate 1.0 -> mask 0 (always). Rate 0.5 -> mask 1 (every other). Rate 0.25 -> mask 3.
fn sampling_mask(rate: f64) -> u64 {
  if rate >= 1.0 {
    return 0;
  }
  if rate <= 0.0 {
    return SAMPLING_NEVER;
  }
  let interval = (1.0 / rate).ceil() as u64;
  interval.next_power_of_two().saturating_sub(1)
}

fn level_to_tracing_level(level: SpanLevel) -> ::tracing::Level {
  match level {
    SpanLevel::Trace => ::tracing::Level::TRACE,
    SpanLevel::Debug => ::tracing::Level::DEBUG,
    SpanLevel::Info => ::tracing::Level::INFO,
    SpanLevel::Warn => ::tracing::Level::WARN,
    SpanLevel::Error => ::tracing::Level::ERROR,
  }
}

fn make_tracing_span(
  options: &SpanOptions,
  context: SpanContext,
  parent_span_id: Option<SpanId>,
) -> Option<::tracing::Span> {
  let parent = parent_span_id.map(|id| id.to_string());
  let span = match level_to_tracing_level(options.level) {
    ::tracing::Level::TRACE => ::tracing::trace_span!(
      "effectful.span",
      otel_name = %options.name,
      otel_trace_id = %context.trace_id,
      otel_span_id = %context.span_id,
      otel_parent_span_id = parent.as_deref().unwrap_or(""),
      otel_trace_flags = %context.trace_flags,
      effectful_status = ::tracing::field::Empty,
      effectful_duration_ns = ::tracing::field::Empty,
      effectful_attributes = ?options.attributes,
    ),
    ::tracing::Level::DEBUG => ::tracing::debug_span!(
      "effectful.span",
      otel_name = %options.name,
      otel_trace_id = %context.trace_id,
      otel_span_id = %context.span_id,
      otel_parent_span_id = parent.as_deref().unwrap_or(""),
      otel_trace_flags = %context.trace_flags,
      effectful_status = ::tracing::field::Empty,
      effectful_duration_ns = ::tracing::field::Empty,
      effectful_attributes = ?options.attributes,
    ),
    ::tracing::Level::INFO => ::tracing::info_span!(
      "effectful.span",
      otel_name = %options.name,
      otel_trace_id = %context.trace_id,
      otel_span_id = %context.span_id,
      otel_parent_span_id = parent.as_deref().unwrap_or(""),
      otel_trace_flags = %context.trace_flags,
      effectful_status = ::tracing::field::Empty,
      effectful_duration_ns = ::tracing::field::Empty,
      effectful_attributes = ?options.attributes,
    ),
    ::tracing::Level::WARN => ::tracing::warn_span!(
      "effectful.span",
      otel_name = %options.name,
      otel_trace_id = %context.trace_id,
      otel_span_id = %context.span_id,
      otel_parent_span_id = parent.as_deref().unwrap_or(""),
      otel_trace_flags = %context.trace_flags,
      effectful_status = ::tracing::field::Empty,
      effectful_duration_ns = ::tracing::field::Empty,
      effectful_attributes = ?options.attributes,
    ),
    ::tracing::Level::ERROR => ::tracing::error_span!(
      "effectful.span",
      otel_name = %options.name,
      otel_trace_id = %context.trace_id,
      otel_span_id = %context.span_id,
      otel_parent_span_id = parent.as_deref().unwrap_or(""),
      otel_trace_flags = %context.trace_flags,
      effectful_status = ::tracing::field::Empty,
      effectful_duration_ns = ::tracing::field::Empty,
      effectful_attributes = ?options.attributes,
    ),
  };
  Some(span)
}

fn make_bridge_only_span(options: &SpanOptions) -> Option<::tracing::Span> {
  if !matches!(tracing_mode(), TracingMode::BridgeOnly) {
    return None;
  }
  let span = match level_to_tracing_level(options.level) {
    ::tracing::Level::TRACE => ::tracing::trace_span!("effectful.span", otel_name = %options.name),
    ::tracing::Level::DEBUG => ::tracing::debug_span!("effectful.span", otel_name = %options.name),
    ::tracing::Level::INFO => ::tracing::info_span!("effectful.span", otel_name = %options.name),
    ::tracing::Level::WARN => ::tracing::warn_span!("effectful.span", otel_name = %options.name),
    ::tracing::Level::ERROR => ::tracing::error_span!("effectful.span", otel_name = %options.name),
  };
  Some(span)
}

fn emit_tracing_span_event(
  span: &Option<::tracing::Span>,
  name: &str,
  attributes: &EffectHashMap<String, SpanAttributeValue>,
) {
  if let Some(span) = span {
    span.in_scope(|| {
      ::tracing::event!(
        ::tracing::Level::INFO,
        effectful_event = %name,
        effectful_event_attributes = ?attributes,
      );
    });
  }
}

fn counter_to_trace_bytes(counter: u64) -> [u8; 16] {
  let mut bytes = [0; 16];
  let counter_bytes = counter.to_be_bytes();
  bytes[8..].copy_from_slice(&counter_bytes);
  bytes
}

pub(crate) fn fiber_refs() -> Option<&'static TracingFiberRefs> {
  TRACING_FIBER_REFS.get()
}

fn ensure_tracing_fiber_refs() -> &'static TracingFiberRefs {
  TRACING_FIBER_REFS.get_or_init(|| {
    let span_stack = run_blocking(
      FiberRef::make_with(
        Vec::<LogSpan>::new,
        |_parent| Vec::new(),
        |parent, _child| parent.clone(),
      ),
      (),
    )
    .expect("tracing span_stack FiberRef");
    let span_annotations = run_blocking(
      FiberRef::make_with(
        hash_map::empty::<String, SpanAttributeValue>,
        |_parent| hash_map::empty(),
        |parent, _child| parent.clone(),
      ),
      (),
    )
    .expect("tracing span_annotations FiberRef");
    TracingFiberRefs {
      span_stack,
      span_annotations,
    }
  })
}

fn with_state_mut<F>(f: F)
where
  F: FnOnce(&mut TraceState),
{
  let mut guard = trace_state().lock().expect("trace state mutex poisoned");
  if !guard.config.enabled || !guard.config.record_in_memory {
    return;
  }
  f(&mut guard);
}

fn tracing_now() -> Instant {
  let clock_now = {
    let guard = trace_state().lock().expect("trace state mutex poisoned");
    Arc::clone(&guard.clock_now)
  };
  clock_now()
}

fn tracing_context_provider() -> Arc<dyn TraceContextProvider> {
  let guard = trace_state().lock().expect("trace state mutex poisoned");
  Arc::clone(&guard.context_provider)
}

fn tracing_clock_now() -> Arc<dyn Fn() -> Instant + Send + Sync> {
  let guard = trace_state().lock().expect("trace state mutex poisoned");
  Arc::clone(&guard.clock_now)
}

fn make_span_record(
  options: &SpanOptions,
  context: SpanContext,
  parent_span_id: Option<SpanId>,
  started_at: Instant,
) -> SpanRecord {
  SpanRecord {
    name: options.name.to_string(),
    context,
    parent_span_id,
    level: options.level,
    status: SpanStatus::Unset,
    attributes: options.attributes.clone(),
    events: Vec::new(),
    started_at,
    ended_at: None,
  }
}

fn record_span_event(
  span_id: SpanId,
  name: String,
  attributes: EffectHashMap<String, SpanAttributeValue>,
  occurred_at: Instant,
) {
  GlobalTraceCollector.span_event(span_id, name, attributes, occurred_at);
}

fn record_effect_event(event: EffectEvent) {
  GlobalTraceCollector.effect_event(event);
}

/// Installs fiber refs and replaces global trace buffers; clears prior events.
pub fn install_tracing_layer(config: TracingConfig) -> Effect<(), Never, ()> {
  install_tracing_layer_with_context_provider(
    config,
    Arc::new(SequentialTraceContextProvider::new()),
  )
}

/// Installs tracing with a clock-backed time source for deterministic timing tests.
pub fn install_tracing_layer_with_clock<C>(config: TracingConfig, clock: C) -> Effect<(), Never, ()>
where
  C: Clock + Send + Sync + 'static,
{
  install_tracing_layer_with_context_provider_and_clock(
    config,
    Arc::new(SequentialTraceContextProvider::new()),
    clock,
  )
}

/// Installs tracing with a custom trace context provider; intended for deterministic tests and integrations.
pub fn install_tracing_layer_with_context_provider(
  config: TracingConfig,
  context_provider: Arc<dyn TraceContextProvider>,
) -> Effect<(), Never, ()> {
  install_tracing_layer_with_context_provider_and_clock_fn(
    config,
    context_provider,
    Arc::new(Instant::now),
  )
}

/// Installs tracing with custom identity and clock providers.
pub fn install_tracing_layer_with_context_provider_and_clock<C>(
  config: TracingConfig,
  context_provider: Arc<dyn TraceContextProvider>,
  clock: C,
) -> Effect<(), Never, ()>
where
  C: Clock + Send + Sync + 'static,
{
  install_tracing_layer_with_context_provider_and_clock_fn(
    config,
    context_provider,
    Arc::new(move || clock.now()),
  )
}

fn install_tracing_layer_with_context_provider_and_clock_fn(
  config: TracingConfig,
  context_provider: Arc<dyn TraceContextProvider>,
  clock_now: Arc<dyn Fn() -> Instant + Send + Sync>,
) -> Effect<(), Never, ()> {
  Effect::new(move |_env| {
    ensure_tracing_fiber_refs();
    let mut guard = trace_state().lock().expect("trace state mutex poisoned");
    guard.config = config.clone();
    let subscriber_available =
      config.enabled && config.bridge_to_tracing && current_tracing_subscriber_available();
    let mode = TracingMode::from_config(&config, subscriber_available);
    TRACING_MODE.store(mode as u64, Ordering::Relaxed);
    let mask = sampling_mask(config.default_sample_rate);
    SAMPLING_MASK.store(mask, Ordering::Relaxed);
    guard.context_provider = Arc::clone(&context_provider);
    guard.clock_now = Arc::clone(&clock_now);
    guard.effect_events.clear();
    guard.fiber_events.clear();
    guard.spans.clear();
    Ok(())
  })
}

/// Appends an effect lifecycle event when tracing is enabled.
pub fn emit_effect_event(event: EffectEvent) -> Effect<(), Never, ()> {
  Effect::new(move |_env| {
    record_effect_event(event.clone());
    Ok(())
  })
}

/// Appends a fiber lifecycle event when tracing is enabled.
pub fn emit_fiber_event(event: FiberEvent) -> Effect<(), Never, ()> {
  Effect::new(move |_env| {
    GlobalTraceCollector.fiber_event(event.clone());
    Ok(())
  })
}

/// Adds `key` → `value` to the current span’s annotation map (fiber-local).
///
/// Generic over `A`, `E`, and `R` for composition with other `Effect<A, E, R>` graphs. In practice
/// `A` is [`unit`](()) and `E` is [`Never`] (see [`AnnotateCurrentSpanSuccess`],
/// [`AnnotateCurrentSpanErr`]). The body does not read `R`; nested fiber ops still use
/// [`run_blocking`] with `()` where those effects require it.
///
/// Type inference often needs an explicit turbofish, e.g.
/// `annotate_current_span::<(), Never, ()>(key, value)` or `::<(), Never, R>` under a generic env.
pub fn annotate_current_span<A, E, R>(
  key: &'static str,
  value: impl Into<String>,
) -> Effect<A, E, R>
where
  A: AnnotateCurrentSpanSuccess + 'static,
  E: AnnotateCurrentSpanErr + 'static,
  R: 'static,
{
  annotate_current_span_attribute(key, SpanAttributeValue::String(value.into()))
}

/// Adds a typed attribute to the current span.
pub fn annotate_current_span_attribute<A, E, R>(
  key: &'static str,
  value: impl Into<SpanAttributeValue>,
) -> Effect<A, E, R>
where
  A: AnnotateCurrentSpanSuccess + 'static,
  E: AnnotateCurrentSpanErr + 'static,
  R: 'static,
{
  let value = value.into();
  effect!(|_r: &mut R| {
    if !tracing_enabled() {
      return Ok(A::from(()));
    }

    let Some(refs) = fiber_refs() else {
      return Ok(A::from(()));
    };

    // FiberRef ops are `Effect<_, _, ()>` — do not `~` them here: `effect!` lowers `~` to `?`, and
    // those results do not convert to a caller-chosen generic `E`. Drive them with `run_blocking`.
    let stack = run_blocking(refs.span_stack.get(), ()).expect("span_stack get");
    if stack.is_empty() {
      return Ok(A::from(()));
    }

    let span_id = stack.last().expect("non-empty span stack").context.span_id;
    let val = value.clone();
    run_blocking(
      refs
        .span_annotations
        .update(move |m| hash_map::set(&m, key.to_string(), val)),
      (),
    )
    .expect("span_annotations update");

    with_state_mut(|state| {
      if let Some(span) = state
        .spans
        .iter_mut()
        .rev()
        .find(|span| span.context.span_id == span_id)
      {
        span.attributes = hash_map::set(&span.attributes, key.to_string(), value.clone());
      }
    });

    A::from(())
  })
}

/// Records an event on the current span with no attributes.
pub fn emit_current_span_event<R>(name: impl Into<String>) -> Effect<(), Never, R>
where
  R: 'static,
{
  emit_current_span_event_with_attributes(name, EffectHashMap::new())
}

/// Records an event on the current span with typed attributes.
pub fn emit_current_span_event_with_attributes<R>(
  name: impl Into<String>,
  attributes: EffectHashMap<String, SpanAttributeValue>,
) -> Effect<(), Never, R>
where
  R: 'static,
{
  let name = name.into();
  effect!(|_r: &mut R| {
    if !tracing_enabled() {
      return Ok(());
    }

    let Some(refs) = fiber_refs() else {
      return Ok(());
    };

    let stack = run_blocking(refs.span_stack.get(), ()).expect("span_stack get");
    let Some(span) = stack.last() else {
      return Ok(());
    };

    emit_tracing_span_event(&span.tracing_span, &name, &attributes);
    record_span_event(
      span.context.span_id,
      name.clone(),
      attributes.clone(),
      tracing_now(),
    );
    ()
  })
}

/// Runs `effect` inside a named INFO span.
pub fn with_span<A, E, R>(
  effect: Effect<A, E, R>,
  name: impl Into<Cow<'static, str>>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  with_span_options(effect, SpanOptions::new(name))
}

/// Runs `effect` inside a span configured by [`SpanOptions`].
pub fn with_span_options<A, E, R>(effect: Effect<A, E, R>, options: SpanOptions) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  Effect::new_step(move |env| {
    let mode = tracing_mode();
    if mode.is_disabled() || !should_sample(options.sample_rate) {
      return start_effect(effect, env);
    }
    if mode.is_bridge_only() {
      return start_effect(with_span_options_bridge_only(effect, options), env);
    }
    start_effect(with_span_options_enabled(effect, options), env)
  })
}

/// Runs `effect` inside a span recorded by an injected collector.
pub fn with_span_options_collected<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
  collector: Arc<dyn TraceCollector>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  with_span_options_collected_with_parent(effect, options, None, collector)
}

/// Runs `effect` inside a span recorded by an injected collector, using a parsed
/// remote `SpanContext` as the parent when no local span is active.
pub fn with_span_options_collected_with_parent<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
  parent: Option<SpanContext>,
  collector: Arc<dyn TraceCollector>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  with_span_options_collected_with_providers_and_parent(
    effect,
    options,
    parent,
    collector,
    Arc::new(SequentialTraceContextProvider::new()),
    Arc::new(Instant::now),
  )
}

fn with_span_options_collected_with_providers_and_parent<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
  initial_parent_context: Option<SpanContext>,
  collector: Arc<dyn TraceCollector>,
  context_provider: Arc<dyn TraceContextProvider>,
  clock_now: Arc<dyn Fn() -> Instant + Send + Sync>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  with_span_options_enabled_with_collector(
    effect,
    options,
    collector,
    context_provider,
    clock_now,
    false,
    initial_parent_context,
  )
}

#[doc(hidden)]
pub fn __effectful_span_lazy<A, E, R, F, O>(
  body: F,
  options: O,
  sample_rate: Option<f64>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
  F: FnOnce() -> Effect<A, E, R> + 'static,
  O: FnOnce() -> SpanOptions + 'static,
{
  let effect = body();
  Effect::new_step(move |env| {
    let mode = tracing_mode();
    if mode.is_disabled() || !should_sample(sample_rate) {
      return start_effect(effect, env);
    }
    let options = options();
    if mode.is_bridge_only() {
      return start_effect(with_span_options_bridge_only(effect, options), env);
    }
    start_effect(with_span_options_enabled(effect, options), env)
  })
}

#[doc(hidden)]
pub fn __effectful_span_lazy_scoped<A, E, R, F>(
  make: F,
  sample_rate: Option<f64>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
  F: FnOnce(bool) -> (Option<SpanOptions>, Effect<A, E, R>) + 'static,
{
  Effect::new_step(move |env| {
    let mode = tracing_mode();
    let instrument = !mode.is_disabled() && should_sample(sample_rate);
    let (options, effect) = make(instrument);
    let Some(options) = options else {
      return start_effect(effect, env);
    };
    if mode.is_bridge_only() {
      return start_effect(with_span_options_bridge_only(effect, options), env);
    }
    start_effect(with_span_options_enabled(effect, options), env)
  })
}

fn with_span_options_bridge_only<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  Effect::new_program(BridgeOnlySpanProgram {
    source: effect,
    options,
  })
}

struct BridgeOnlySpanProgram<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  source: Effect<A, E, R>,
  options: SpanOptions,
}

impl<A, E, R> ProgramOp<A, E, R> for BridgeOnlySpanProgram<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  fn start(self: Box<Self>, env: &mut R) -> SyncStep<A, E, R> {
    let Self { source, options } = *self;
    if tracing_mode().is_disabled() {
      return start_effect(source, env);
    }
    let Some(span) = make_bridge_only_span(&options) else {
      return start_effect(source, env);
    };
    if span.is_disabled() {
      return start_effect(source, env);
    }
    match span.in_scope(|| start_effect(source, env)) {
      SyncStep::Ready(output) => SyncStep::Ready(output),
      SyncStep::AsyncBorrow(f) => SyncStep::AsyncBorrow(Box::new(move |env| {
        let span = span.clone();
        box_future(async move {
          use ::tracing::Instrument as _;
          start_async_operation(f, env).instrument(span).await
        })
      })),
      SyncStep::AsyncStatic(fut) => SyncStep::AsyncStatic(box_future(async move {
        use ::tracing::Instrument as _;
        fut.instrument(span).await
      })),
      SyncStep::AsyncPoll(mut poller) => SyncStep::AsyncPoll(Box::new(move |env, cx| {
        let entered = span.enter();
        match poller(env, cx) {
          std::task::Poll::Ready(output) => {
            drop(entered);
            std::task::Poll::Ready(output)
          }
          std::task::Poll::Pending => std::task::Poll::Pending,
        }
      })),
    }
  }
}

fn with_span_options_enabled<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  with_span_options_enabled_with_collector(
    effect,
    options,
    Arc::new(GlobalTraceCollector),
    tracing_context_provider(),
    tracing_clock_now(),
    true,
    None,
  )
}

fn with_span_options_enabled_with_collector<A, E, R>(
  effect: Effect<A, E, R>,
  options: SpanOptions,
  collector: Arc<dyn TraceCollector>,
  context_provider: Arc<dyn TraceContextProvider>,
  clock_now: Arc<dyn Fn() -> Instant + Send + Sync>,
  bridge_to_tracing: bool,
  initial_parent_context: Option<SpanContext>,
) -> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  Effect::new_async(move |env: &mut R| {
    let options = options.clone();
    let collector = Arc::clone(&collector);
    let context_provider = Arc::clone(&context_provider);
    let clock_now = Arc::clone(&clock_now);
    box_future(async move {
      if bridge_to_tracing && tracing_mode().is_disabled() {
        return effect.run(env).await;
      }
      let refs = ensure_tracing_fiber_refs().clone();
      let stack = run_blocking(refs.span_stack.get(), ()).expect("span_stack get");
      let parent_context = stack
        .last()
        .map(|span| span.context)
        .or(initial_parent_context);
      let context = match parent_context {
        Some(parent) => context_provider.child_context(&parent),
        None => context_provider.root_context(),
      };
      let started_at = clock_now();
      let parent_span_id = parent_context.map(|parent| parent.span_id);
      let tracing_span = if bridge_to_tracing {
        make_tracing_span(&options, context, parent_span_id)
      } else {
        None
      };
      collector.span_started(make_span_record(
        &options,
        context,
        parent_span_id,
        started_at,
      ));
      collector.effect_event(EffectEvent::Start {
        span: options.name.to_string(),
      });

      let span_name_for_push = options.name.clone();
      let tracing_span_for_push = tracing_span.clone();
      run_blocking(
        refs.span_stack.update(move |mut v| {
          v.push(LogSpan {
            name: span_name_for_push,
            context,
            tracing_span: tracing_span_for_push,
          });
          v
        }),
        (),
      )
      .expect("span_stack push");
      let mut span_stack_guard = SpanStackPopGuard::new(refs.clone());

      let initial_attributes = options.attributes.clone();
      let refs_for_inner = refs.clone();
      let span_name_inner = options.name.to_string();
      let tracing_span_for_inner = tracing_span.clone();
      let inner = Effect::new_async(move |env: &mut R| {
        let span_name = span_name_inner.clone();
        let refs = refs_for_inner.clone();
        let tracing_span = tracing_span_for_inner.clone();
        let collector = Arc::clone(&collector);
        let clock_now = Arc::clone(&clock_now);
        box_future(async move {
          let out = effect.run(env).await;
          if !bridge_to_tracing || tracing_enabled() {
            let attributes = run_blocking(refs.span_annotations.get(), ())
              .expect("span_annotations get for flush");
            let status = match &out {
              Ok(_) => SpanStatus::Ok,
              Err(_) => SpanStatus::Error,
            };
            let ended_at = clock_now();
            if let Some(span) = &tracing_span {
              let status_label = match status {
                SpanStatus::Unset => "unset",
                SpanStatus::Ok => "ok",
                SpanStatus::Error => "error",
              };
              let duration_ns =
                u64::try_from(ended_at.duration_since(started_at).as_nanos()).unwrap_or(u64::MAX);
              span.record("effectful_status", status_label);
              span.record("effectful_duration_ns", duration_ns);
            }
            collector.span_ended(context.span_id, attributes, status, ended_at);
          }
          let event = match &out {
            Ok(_) => EffectEvent::Success {
              span: span_name.clone(),
            },
            Err(_) => EffectEvent::Failure {
              span: span_name.clone(),
            },
          };
          collector.effect_event(event);
          out
        })
      });

      let instrumented = refs
        .span_annotations
        .locally(initial_attributes, inner)
        .run(env);
      let out = if let Some(span) = tracing_span {
        use ::tracing::Instrument as _;
        instrumented.instrument(span).await
      } else {
        instrumented.await
      };

      span_stack_guard.pop();

      out
    })
  })
}

impl<A, E, R> Effect<A, E, R>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  /// Runs this effect inside a named INFO span.
  #[inline]
  pub fn with_span(self, name: impl Into<Cow<'static, str>>) -> Effect<A, E, R> {
    with_span(self, name)
  }

  /// Runs this effect inside a span configured by [`SpanOptions`].
  #[inline]
  pub fn with_span_options(self, options: SpanOptions) -> Effect<A, E, R> {
    with_span_options(self, options)
  }
}

/// Clones the current global trace buffers (lock held briefly).
pub fn snapshot_tracing() -> TracingSnapshot {
  let guard = trace_state().lock().expect("trace state mutex poisoned");
  TracingSnapshot {
    effect_events: guard.effect_events.clone(),
    fiber_events: guard.fiber_events.clone(),
    spans: guard.spans.clone(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::collections::hash_map;
  use crate::scheduling::TestClock;
  use crate::{fail, runtime::run_blocking, succeed};
  use rstest::rstest;
  use std::sync::{Arc, Mutex, OnceLock};
  use std::time::{Duration, Instant};

  static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

  fn test_lock() -> std::sync::MutexGuard<'static, ()> {
    TEST_LOCK
      .get_or_init(|| Mutex::new(()))
      .lock()
      .expect("test lock mutex poisoned")
  }

  #[derive(Clone, Default)]
  struct RecordingTraceCollector {
    snapshot: Arc<Mutex<TracingSnapshot>>,
  }

  impl RecordingTraceCollector {
    fn snapshot(&self) -> TracingSnapshot {
      self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned")
        .clone()
    }
  }

  impl TraceCollector for RecordingTraceCollector {
    fn span_started(&self, span: SpanRecord) {
      self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned")
        .spans
        .push(span);
    }

    fn span_ended(
      &self,
      span_id: SpanId,
      attributes: EffectHashMap<String, SpanAttributeValue>,
      status: SpanStatus,
      ended_at: Instant,
    ) {
      let mut snapshot = self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned");
      if let Some(span) = snapshot
        .spans
        .iter_mut()
        .rev()
        .find(|span| span.context.span_id == span_id)
      {
        span.attributes = attributes;
        span.status = status;
        span.ended_at = Some(ended_at);
      }
    }

    fn span_event(
      &self,
      span_id: SpanId,
      name: String,
      attributes: EffectHashMap<String, SpanAttributeValue>,
      occurred_at: Instant,
    ) {
      let mut snapshot = self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned");
      if let Some(span) = snapshot
        .spans
        .iter_mut()
        .rev()
        .find(|span| span.context.span_id == span_id)
      {
        span.events.push(SpanEvent {
          name,
          attributes,
          occurred_at,
        });
      }
    }

    fn effect_event(&self, event: EffectEvent) {
      self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned")
        .effect_events
        .push(event);
    }

    fn fiber_event(&self, event: FiberEvent) {
      self
        .snapshot
        .lock()
        .expect("recording collector mutex poisoned")
        .fiber_events
        .push(event);
    }
  }

  mod with_span_events {
    use super::*;

    #[test]
    fn with_span_options_collected_when_effect_succeeds_records_lifecycle() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
      let collector = RecordingTraceCollector::default();
      let eff = with_span_options_collected(
        succeed::<u32, (), ()>(7),
        SpanOptions::new("injected.span").with_attribute("path", "injected"),
        Arc::new(collector.clone()),
      );

      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(7));

      let snapshot = collector.snapshot();
      assert_eq!(snapshot.spans.len(), 1);
      let span = &snapshot.spans[0];
      assert_eq!(span.name, "injected.span");
      assert_eq!(span.status, SpanStatus::Ok);
      assert_eq!(span.level, SpanLevel::Info);
      assert_eq!(
        span.attributes.get("path"),
        Some(&SpanAttributeValue::String("injected".to_string()))
      );
      assert!(span.ended_at.is_some());
      assert_eq!(
        snapshot.effect_events,
        vec![
          EffectEvent::Start {
            span: "injected.span".to_string()
          },
          EffectEvent::Success {
            span: "injected.span".to_string()
          }
        ]
      );
      assert!(snapshot_tracing().spans.is_empty());
    }

    #[test]
    fn with_span_when_effect_succeeds_records_start_and_success_events() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(succeed::<u32, (), ()>(7), "test.span");
      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(7));

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|span| span.name == "test.span")
        .expect("span record");
      assert_eq!(span.status, SpanStatus::Ok);
      assert_eq!(span.level, SpanLevel::Info);
      assert!(span.ended_at.is_some());
      assert!(span.duration().is_some());
      assert_eq!(
        snapshot.effect_events,
        vec![
          EffectEvent::Start {
            span: "test.span".to_string()
          },
          EffectEvent::Success {
            span: "test.span".to_string()
          }
        ]
      );
    }

    #[test]
    fn with_span_when_effect_fails_records_start_and_failure_events() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(fail::<(), &'static str, ()>("boom"), "failure.span");
      let out = run_blocking(eff, ());
      assert_eq!(out, Err("boom"));

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|span| span.name == "failure.span")
        .expect("span record");
      assert_eq!(span.status, SpanStatus::Error);
      assert_eq!(
        snapshot.effect_events,
        vec![
          EffectEvent::Start {
            span: "failure.span".to_string()
          },
          EffectEvent::Failure {
            span: "failure.span".to_string()
          }
        ]
      );
    }

    #[test]
    fn nested_spans_record_parent_child_context() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(with_span(succeed::<(), (), ()>(()), "inner"), "outer");
      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(()));

      let snapshot = snapshot_tracing();
      let outer = snapshot
        .spans
        .iter()
        .find(|span| span.name == "outer")
        .expect("outer span");
      let inner = snapshot
        .spans
        .iter()
        .find(|span| span.name == "inner")
        .expect("inner span");
      assert_eq!(outer.parent_span_id, None);
      assert_eq!(inner.parent_span_id, Some(outer.context.span_id));
      assert_eq!(inner.context.trace_id, outer.context.trace_id);
    }

    #[test]
    fn method_with_span_records_span() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = succeed::<u8, (), ()>(1).with_span_options(
        SpanOptions::new("method.span")
          .with_level(SpanLevel::Debug)
          .with_attribute("static", "yes"),
      );
      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(1));

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|span| span.name == "method.span")
        .expect("method span");
      assert_eq!(span.level, SpanLevel::Debug);
      assert_eq!(
        span.attributes.get("static"),
        Some(&SpanAttributeValue::String("yes".to_string()))
      );
    }

    #[test]
    fn installed_clock_controls_span_timing() {
      let _guard = test_lock();
      let start = Instant::now();
      let clock = TestClock::new(start);
      let clock_for_effect = clock.clone();
      let _ = run_blocking(
        install_tracing_layer_with_clock(TracingConfig::enabled(), clock),
        (),
      );
      let eff = with_span(
        Effect::new(move |_env: &mut ()| {
          clock_for_effect.advance(Duration::from_millis(5));
          Ok::<(), ()>(())
        }),
        "clock.span",
      );
      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(()));

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|span| span.name == "clock.span")
        .expect("clock span");
      assert_eq!(span.started_at, start);
      assert_eq!(span.duration(), Some(Duration::from_millis(5)));
    }

    #[test]
    fn with_span_options_collected_with_parent_uses_parsed_traceparent_without_installing_layer() {
      let _guard = test_lock();
      let parsed =
        SpanContext::from_traceparent("00-01010101010101010101010101010101-0202020202020202-01")
          .expect("valid traceparent");
      let collector = RecordingTraceCollector::default();
      let eff = with_span_options_collected_with_parent(
        succeed::<u32, (), ()>(7),
        SpanOptions::new("injected.parent.span"),
        Some(parsed),
        Arc::new(collector.clone()),
      );

      let out = run_blocking(eff, ());
      assert_eq!(out, Ok(7));

      let snapshot = collector.snapshot();
      assert_eq!(snapshot.spans.len(), 1);
      let span = &snapshot.spans[0];
      assert_eq!(span.name, "injected.parent.span");
      assert_eq!(span.context.trace_id, parsed.trace_id);
      assert_eq!(span.parent_span_id, Some(parsed.span_id));
      assert_ne!(span.context.span_id, parsed.span_id);
      assert_eq!(span.context.trace_flags, parsed.trace_flags);
      assert_eq!(span.status, SpanStatus::Ok);
    }

    #[test]
    fn nested_spans_with_parsed_parent_use_fiber_stack_for_inner_parent() {
      let _guard = test_lock();
      let parsed =
        SpanContext::from_traceparent("00-01010101010101010101010101010101-0202020202020202-01")
          .expect("valid traceparent");
      let collector = RecordingTraceCollector::default();
      let inner = with_span_options_collected_with_parent(
        succeed::<(), (), ()>(()),
        SpanOptions::new("inner.collected"),
        Some(parsed),
        Arc::new(collector.clone()),
      );
      let outer = with_span_options_collected_with_parent(
        inner,
        SpanOptions::new("outer.collected"),
        Some(parsed),
        Arc::new(collector.clone()),
      );

      let out = run_blocking(outer, ());
      assert_eq!(out, Ok(()));

      let snapshot = collector.snapshot();
      assert_eq!(snapshot.spans.len(), 2);
      let outer_span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "outer.collected")
        .expect("outer span");
      let inner_span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "inner.collected")
        .expect("inner span");
      assert_eq!(outer_span.parent_span_id, Some(parsed.span_id));
      assert_eq!(inner_span.parent_span_id, Some(outer_span.context.span_id));
      assert_eq!(outer_span.context.trace_id, parsed.trace_id);
      assert_eq!(inner_span.context.trace_id, parsed.trace_id);
    }

    #[test]
    fn collected_nested_spans_with_parsed_traceparent_do_not_need_tracing_bridge() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
      let parsed =
        SpanContext::from_traceparent("00-01010101010101010101010101010101-0202020202020202-01")
          .expect("valid traceparent");
      let collector = RecordingTraceCollector::default();
      let inner = with_span_options_collected(
        succeed::<(), (), ()>(()),
        SpanOptions::new("inner.no_bridge"),
        Arc::new(collector.clone()),
      );
      let outer = with_span_options_collected_with_parent(
        inner,
        SpanOptions::new("outer.no_bridge"),
        Some(parsed),
        Arc::new(collector.clone()),
      );

      let out = run_blocking(outer, ());
      assert_eq!(out, Ok(()));

      let snapshot = collector.snapshot();
      assert_eq!(snapshot.spans.len(), 2);
      let outer_span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "outer.no_bridge")
        .expect("outer span");
      let inner_span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "inner.no_bridge")
        .expect("inner span");

      assert!(outer_span.ended_at.is_some());
      assert_eq!(outer_span.status, SpanStatus::Ok);
      assert!(inner_span.ended_at.is_some());
      assert_eq!(inner_span.status, SpanStatus::Ok);

      assert_eq!(outer_span.parent_span_id, Some(parsed.span_id));
      assert_eq!(outer_span.context.trace_id, parsed.trace_id);
      assert_eq!(outer_span.context.trace_flags, parsed.trace_flags);

      assert_eq!(inner_span.parent_span_id, Some(outer_span.context.span_id));
      assert_eq!(inner_span.context.trace_id, parsed.trace_id);
      assert_eq!(inner_span.context.trace_flags, parsed.trace_flags);

      assert_eq!(
        snapshot.effect_events,
        vec![
          EffectEvent::Start {
            span: "outer.no_bridge".to_string(),
          },
          EffectEvent::Start {
            span: "inner.no_bridge".to_string(),
          },
          EffectEvent::Success {
            span: "inner.no_bridge".to_string(),
          },
          EffectEvent::Success {
            span: "outer.no_bridge".to_string(),
          },
        ]
      );
      assert!(snapshot_tracing().spans.is_empty());
      assert!(snapshot_tracing().effect_events.is_empty());
    }
  }

  mod tracing_bridge {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tracing::Subscriber;
    use tracing::field::{Field, Visit};
    use tracing::span::{Attributes, Id};
    use tracing_subscriber::layer::{Context, SubscriberExt};
    use tracing_subscriber::{Layer, Registry};

    #[derive(Clone, Default)]
    struct RecordingLayer {
      spans: Arc<Mutex<Vec<String>>>,
      events: Arc<Mutex<Vec<String>>>,
    }

    impl<S> Layer<S> for RecordingLayer
    where
      S: Subscriber,
    {
      fn on_new_span(&self, attrs: &Attributes<'_>, _id: &Id, _ctx: Context<'_, S>) {
        let mut visitor = BridgeVisitor::default();
        attrs.record(&mut visitor);
        if let Some(name) = visitor.otel_name {
          self.spans.lock().expect("spans mutex poisoned").push(name);
        }
      }

      fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        let mut visitor = BridgeVisitor::default();
        event.record(&mut visitor);
        if let Some(name) = visitor.effectful_event {
          self
            .events
            .lock()
            .expect("events mutex poisoned")
            .push(name);
        }
      }
    }

    #[derive(Default)]
    struct BridgeVisitor {
      otel_name: Option<String>,
      effectful_event: Option<String>,
    }

    impl Visit for BridgeVisitor {
      fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let value = format!("{value:?}");
        if field.name() == "otel_name" {
          self.otel_name = Some(value.trim_matches('"').to_string());
        } else if field.name() == "effectful_event" {
          self.effectful_event = Some(value.trim_matches('"').to_string());
        }
      }
    }

    #[test]
    fn bridge_emits_rust_tracing_spans_and_events() {
      let _guard = test_lock();
      let layer = RecordingLayer::default();
      let subscriber = Registry::default().with(layer.clone());

      tracing::subscriber::with_default(subscriber, || {
        let _ = run_blocking(
          install_tracing_layer(TracingConfig::enabled_with_tracing_bridge()),
          (),
        );
        let eff = with_span(
          emit_current_span_event::<()>("domain.loaded"),
          "bridge.span",
        );
        let out = run_blocking(eff, ());
        assert_eq!(out, Ok(()));
      });

      assert_eq!(
        layer.spans.lock().expect("spans mutex poisoned").as_slice(),
        &["bridge.span".to_string()]
      );
      assert_eq!(
        layer
          .events
          .lock()
          .expect("events mutex poisoned")
          .as_slice(),
        &["domain.loaded".to_string()]
      );
    }

    #[test]
    fn bridge_only_emits_without_snapshotting() {
      let _guard = test_lock();
      let layer = RecordingLayer::default();
      let subscriber = Registry::default().with(layer.clone());

      tracing::subscriber::with_default(subscriber, || {
        let _ = run_blocking(
          install_tracing_layer(TracingConfig::tracing_bridge_only()),
          (),
        );
        let out = run_blocking(with_span(succeed::<u8, (), ()>(1), "bridge.only"), ());
        assert_eq!(out, Ok(1));
      });

      assert_eq!(
        layer.spans.lock().expect("spans mutex poisoned").as_slice(),
        &["bridge.only".to_string()]
      );
      let snapshot = snapshot_tracing();
      assert!(snapshot.spans.is_empty());
      assert!(snapshot.effect_events.is_empty());
    }
  }

  mod hooks_and_config {
    use super::*;

    #[test]
    fn annotation_and_fiber_event_hooks_when_enabled_record_data() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(
        annotate_current_span::<(), Never, ()>("market", "SOL-PERP").flat_map(|_| {
          emit_fiber_event(FiberEvent::Spawn {
            fiber_id: "fiber-1".to_string(),
          })
        }),
        "annotated.span",
      );
      let _ = run_blocking(eff, ());

      let snapshot = snapshot_tracing();
      assert_eq!(snapshot.fiber_events.len(), 1);
      let span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "annotated.span")
        .expect("span should be present");
      assert_eq!(
        span.attributes.get("market"),
        Some(&SpanAttributeValue::String("SOL-PERP".to_string()))
      );
    }

    #[test]
    fn typed_attributes_are_recorded() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(
        annotate_current_span_attribute::<(), Never, ()>("retry", 2_i32)
          .flat_map(|_| annotate_current_span_attribute::<(), Never, ()>("cached", true)),
        "typed.span",
      );
      let _ = run_blocking(eff, ());

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "typed.span")
        .expect("span should be present");
      assert_eq!(
        span.attributes.get("retry"),
        Some(&SpanAttributeValue::I64(2))
      );
      assert_eq!(
        span.attributes.get("cached"),
        Some(&SpanAttributeValue::Bool(true))
      );
    }

    #[test]
    fn current_span_events_are_recorded() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let attributes = hash_map::set(
        &hash_map::empty(),
        "rows".to_string(),
        SpanAttributeValue::I64(3),
      );
      let eff = with_span(
        emit_current_span_event_with_attributes::<()>("loaded", attributes),
        "event.span",
      );
      let _ = run_blocking(eff, ());

      let snapshot = snapshot_tracing();
      let span = snapshot
        .spans
        .iter()
        .find(|s| s.name == "event.span")
        .expect("span should be present");
      assert_eq!(span.events.len(), 1);
      assert_eq!(span.events[0].name, "loaded");
      assert_eq!(
        span.events[0].attributes.get("rows"),
        Some(&SpanAttributeValue::I64(3))
      );
    }

    #[rstest]
    #[case::effect_event(0)]
    #[case::fiber_event(1)]
    fn emit_hooks_when_tracing_disabled_do_not_record_events(#[case] mode: u8) {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
      if mode == 0 {
        let _ = run_blocking(
          emit_effect_event(EffectEvent::Start {
            span: "disabled.span".to_string(),
          }),
          (),
        );
      } else {
        let _ = run_blocking(
          emit_fiber_event(FiberEvent::Spawn {
            fiber_id: "fiber-disabled".to_string(),
          }),
          (),
        );
      }
      let snapshot = snapshot_tracing();
      assert!(snapshot.effect_events.is_empty());
      assert!(snapshot.fiber_events.is_empty());
      assert!(snapshot.spans.is_empty());
    }

    #[test]
    fn annotate_current_span_when_no_active_span_is_present_is_noop() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let _ = run_blocking(annotate_current_span::<(), Never, ()>("k", "v"), ());
      let snapshot = snapshot_tracing();
      assert!(snapshot.spans.is_empty());
      assert!(snapshot.effect_events.is_empty());
      assert!(snapshot.fiber_events.is_empty());
    }

    #[test]
    fn tracing_config_enabled_constructor_sets_enabled_true() {
      let cfg = TracingConfig::enabled();
      assert!(cfg.enabled);
      assert!(cfg.record_in_memory);
    }

    #[test]
    fn span_decides_enabled_state_when_run_not_when_constructed() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());
      let eff = with_span(succeed::<u8, (), ()>(1), "late.enabled");
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());

      let out = run_blocking(eff, ());

      assert_eq!(out, Ok(1));
      let snapshot = snapshot_tracing();
      assert!(
        snapshot
          .spans
          .iter()
          .any(|span| span.name == "late.enabled")
      );
    }

    #[test]
    fn span_decides_disabled_state_when_run_not_when_constructed() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(succeed::<u8, (), ()>(1), "late.disabled");
      let _ = run_blocking(install_tracing_layer(TracingConfig::default()), ());

      let out = run_blocking(eff, ());

      assert_eq!(out, Ok(1));
      let snapshot = snapshot_tracing();
      assert!(snapshot.spans.is_empty());
    }

    #[test]
    fn zero_sample_rate_records_no_spans() {
      let _guard = test_lock();
      let _ = run_blocking(
        install_tracing_layer(TracingConfig {
          enabled: true,
          bridge_to_tracing: false,
          record_in_memory: true,
          default_sample_rate: 0.0,
        }),
        (),
      );

      let out = run_blocking(with_span(succeed::<u8, (), ()>(1), "never.sampled"), ());

      assert_eq!(out, Ok(1));
      assert!(snapshot_tracing().spans.is_empty());
    }

    #[test]
    fn sampling_mask_rounds_down_to_power_of_two_cadence() {
      assert_eq!(sampling_mask(1.0), 0);
      assert_eq!(sampling_mask(0.5), 1);
      assert_eq!(sampling_mask(0.3), 3);
      assert_eq!(sampling_mask(0.25), 3);
      assert_eq!(sampling_mask(0.0), SAMPLING_NEVER);
    }

    #[test]
    fn tracing_snapshot_attributes_preserved_across_clone() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let eff = with_span(
        annotate_current_span::<(), Never, ()>("market", "SOL-PERP"),
        "clone.span",
      );
      let _ = run_blocking(eff, ());

      let snap = snapshot_tracing();
      let mut snap_clone = snap.clone();
      let span = snap_clone
        .spans
        .iter_mut()
        .find(|s| s.name == "clone.span")
        .expect("span recorded");
      span.attributes = hash_map::set(
        &span.attributes,
        "market".to_string(),
        SpanAttributeValue::String("edited".to_string()),
      );

      let orig = snap
        .spans
        .iter()
        .find(|s| s.name == "clone.span")
        .expect("span in original snapshot");
      assert_eq!(
        orig.attributes.get("market"),
        Some(&SpanAttributeValue::String("SOL-PERP".to_string()))
      );
      assert_eq!(
        snap_clone
          .spans
          .iter()
          .find(|s| s.name == "clone.span")
          .expect("span in clone")
          .attributes
          .get("market"),
        Some(&SpanAttributeValue::String("edited".to_string()))
      );
    }
  }

  mod fiber_local_tracing {
    use super::*;
    use crate::concurrency::fiber_ref::with_fiber_id;
    use crate::runtime::FiberId;

    #[test]
    fn annotation_isolated_per_fiber() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let ef_a = with_span(
        annotate_current_span::<(), Never, ()>("k", "fiber-a"),
        "span.a",
      );
      let ef_b = with_span(
        annotate_current_span::<(), Never, ()>("k", "fiber-b"),
        "span.b",
      );
      with_fiber_id(FiberId::fresh(), || {
        let _ = run_blocking(ef_a, ());
      });
      with_fiber_id(FiberId::fresh(), || {
        let _ = run_blocking(ef_b, ());
      });
      let snap = snapshot_tracing();
      let sa = snap
        .spans
        .iter()
        .find(|s| s.name == "span.a")
        .expect("span.a");
      let sb = snap
        .spans
        .iter()
        .find(|s| s.name == "span.b")
        .expect("span.b");
      assert_eq!(
        sa.attributes.get("k"),
        Some(&SpanAttributeValue::String("fiber-a".to_string()))
      );
      assert_eq!(
        sb.attributes.get("k"),
        Some(&SpanAttributeValue::String("fiber-b".to_string()))
      );
    }

    #[test]
    fn span_stack_not_shared_between_fibers() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let refs = fiber_refs().expect("refs").clone();
      let id_a = FiberId::fresh();
      let id_b = FiberId::fresh();
      with_fiber_id(id_a, || {
        run_blocking(
          refs.span_stack.update(|mut v| {
            v.push(LogSpan {
              name: "only-a".into(),
              context: SpanContext {
                trace_id: TraceId::from_bytes([0; 16]),
                span_id: SpanId::from_bytes([1; 8]),
                trace_flags: TraceFlags::DEFAULT,
              },
              tracing_span: None,
            });
            v
          }),
          (),
        )
        .expect("push stack");
      });
      with_fiber_id(id_b, || {
        let len = run_blocking(refs.span_stack.get(), ())
          .expect("get stack")
          .len();
        assert_eq!(len, 0, "B should not see A's stack");
      });
      with_fiber_id(id_a, || {
        let len = run_blocking(refs.span_stack.get(), ())
          .expect("get stack a")
          .len();
        assert_eq!(len, 1);
      });
    }

    #[test]
    fn with_span_pushes_then_pops() {
      let _guard = test_lock();
      let _ = run_blocking(install_tracing_layer(TracingConfig::enabled()), ());
      let refs = fiber_refs().expect("refs").clone();
      let eff = with_span(with_span(succeed::<(), (), ()>(()), "inner"), "outer");
      let _ = run_blocking(eff, ());
      let len = run_blocking(refs.span_stack.get(), ())
        .expect("stack len")
        .len();
      assert_eq!(len, 0);
    }
  }
}
