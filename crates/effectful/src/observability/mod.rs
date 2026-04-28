//! **Stratum 15 — Observability**
//!
//! Structured metrics and tracing instrumentation, built from Strata 0–14.
//!
//! | Submodule | Provides | Depends on |
//! |-----------|----------|------------|
//! | [`metric`] | [`Metric`], counters/gauges/histograms | Stratum 14 (`collections::hash_map`), Stratum 10 (`scheduling::duration`), Stratum 2 (`kernel`) |
//! | [`tracing`] | [`TracingConfig`], span/fiber hooks | Stratum 7 (`concurrency::fiber_ref`), Stratum 14 (`collections::hash_map`), Stratum 2 (`kernel`) |

pub mod metric;
pub mod traceparent;
pub mod tracing;

pub use metric::{Metric, make as metric_make};
pub use traceparent::{SpanContext, SpanId, TraceFlags, TraceId, TraceParentParseError};
pub use tracing::{
  __effectful_span_lazy, __effectful_span_lazy_scoped, AnnotateCurrentSpanErr,
  AnnotateCurrentSpanSuccess, EffectEvent, FiberEvent, GlobalTraceCollector, LogSpan,
  SequentialTraceContextProvider, SpanAttributeValue, SpanEvent, SpanLevel,
  SpanOptions, SpanRecord, SpanStatus, TraceCollector, TraceContextProvider,
  TracingConfig, TracingFiberRefs, TracingSnapshot, annotate_current_span,
  annotate_current_span_attribute, emit_current_span_event,
  emit_current_span_event_with_attributes, emit_effect_event, emit_fiber_event,
  install_tracing_layer, install_tracing_layer_with_clock,
  install_tracing_layer_with_context_provider,
  install_tracing_layer_with_context_provider_and_clock, snapshot_tracing, tracing_enabled,
  with_span, with_span_options, with_span_options_collected,
};
