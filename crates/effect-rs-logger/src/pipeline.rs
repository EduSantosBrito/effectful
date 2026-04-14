//! Composable log backends: tracing, JSON ([`serde_json`]), and plain structured lines.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write;
use std::sync::{Arc, Mutex, RwLock};

use ::effect::EffectHashMap;
use serde::Serialize;

use crate::{EffectLoggerError, LogLevel};

/// One log event passed to each backend in a [`CompositeLogBackend`].
#[derive(Debug, Clone)]
pub struct LogRecord<'a> {
  /// Severity of this line.
  pub level: LogLevel,
  /// Human-readable message body.
  pub message: Cow<'a, str>,
  /// Structured key/value fields merged into the formatted line or JSON row.
  pub annotations: EffectHashMap<String, String>,
  /// Active span names from outermost to innermost (for nesting display).
  pub spans: Vec<String>,
}

/// Sink for [`LogRecord`] values (tracing, JSON file, tests, etc.).
pub trait LogBackend: Send + Sync {
  /// Deliver one record to this sink (e.g. write a line, call `tracing`, …).
  fn emit(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError>;
}

/// Effect.ts-style composable logger: [`Self::add`], [`Self::replace`], [`Self::remove`].
///
/// `Message` and `Output` from the gap doc map to [`LogRecord::message`] and
/// `Result<(), EffectLoggerError>` respectively.
pub trait Logger: Send + Sync {
  /// Append a backend to the ordered fan-out list.
  fn add(&self, backend: Arc<dyn LogBackend>) -> Result<(), EffectLoggerError>;
  /// Swap the backend at `idx` without changing list length.
  fn replace(&self, idx: usize, backend: Arc<dyn LogBackend>) -> Result<(), EffectLoggerError>;
  /// Remove the backend at `idx`, shifting later entries down.
  fn remove(&self, idx: usize) -> Result<(), EffectLoggerError>;
}

/// Thread-safe list of backends; also implements [`LogBackend`] by fan-out.
pub struct CompositeLogBackend {
  backends: RwLock<Vec<Arc<dyn LogBackend>>>,
}

impl Default for CompositeLogBackend {
  fn default() -> Self {
    Self::new()
  }
}

impl CompositeLogBackend {
  /// Empty backend list; use [`Logger::add`] to register sinks.
  pub fn new() -> Self {
    Self {
      backends: RwLock::new(Vec::new()),
    }
  }

  /// Emit to every registered backend in order.
  pub fn emit_all(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError> {
    let bs: Vec<_> = self
      .backends
      .read()
      .map_err(|e| EffectLoggerError::Sink(format!("composite read lock: {e}")))?
      .clone();
    for b in bs {
      b.emit(rec)?;
    }
    Ok(())
  }
}

impl Logger for CompositeLogBackend {
  fn add(&self, backend: Arc<dyn LogBackend>) -> Result<(), EffectLoggerError> {
    self
      .backends
      .write()
      .map_err(|e| EffectLoggerError::Sink(format!("composite write lock: {e}")))?
      .push(backend);
    Ok(())
  }

  fn replace(&self, idx: usize, backend: Arc<dyn LogBackend>) -> Result<(), EffectLoggerError> {
    let mut g = self
      .backends
      .write()
      .map_err(|e| EffectLoggerError::Sink(format!("composite write lock: {e}")))?;
    if idx >= g.len() {
      return Err(EffectLoggerError::Sink(format!(
        "logger replace: index {idx} out of bounds (len {})",
        g.len()
      )));
    }
    g[idx] = backend;
    Ok(())
  }

  fn remove(&self, idx: usize) -> Result<(), EffectLoggerError> {
    let mut g = self
      .backends
      .write()
      .map_err(|e| EffectLoggerError::Sink(format!("composite write lock: {e}")))?;
    if idx >= g.len() {
      return Err(EffectLoggerError::Sink(format!(
        "logger remove: index {idx} out of bounds (len {})",
        g.len()
      )));
    }
    g.remove(idx);
    Ok(())
  }
}

impl LogBackend for CompositeLogBackend {
  fn emit(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError> {
    self.emit_all(rec)
  }
}

fn format_tracing_line(rec: &LogRecord<'_>) -> String {
  let mut full = String::new();
  if !rec.spans.is_empty() {
    let _ = write!(&mut full, "[{}] ", rec.spans.join(" > "));
  }
  full.push_str(rec.message.as_ref());
  for (k, v) in rec.annotations.iter() {
    let _ = write!(&mut full, " {k}={v}");
  }
  full
}

/// Forwards to the `tracing` crate (same levels as [`crate::EffectLogger`] legacy path).
#[derive(Debug, Default, Clone, Copy)]
pub struct TracingLogBackend;

impl LogBackend for TracingLogBackend {
  fn emit(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError> {
    let line = format_tracing_line(rec);
    match rec.level {
      LogLevel::Trace => tracing::trace!("{}", line),
      LogLevel::Debug => tracing::debug!("{}", line),
      LogLevel::Info => tracing::info!("{}", line),
      LogLevel::Warn => tracing::warn!("{}", line),
      LogLevel::Error | LogLevel::Fatal => tracing::error!("{}", line),
      LogLevel::None => {}
    }
    Ok(())
  }
}

/// One JSON object per line (`serde_json`), for files or test buffers.
#[derive(Clone)]
pub struct JsonLogBackend<W: Write + Send + 'static> {
  writer: Arc<Mutex<W>>,
}

impl<W: Write + Send + 'static> JsonLogBackend<W> {
  /// Wrap `writer`; each [`LogBackend::emit`] appends one JSON object and newline.
  pub fn new(writer: W) -> Self {
    Self {
      writer: Arc::new(Mutex::new(writer)),
    }
  }

  /// Clone the shared writer handle (e.g. read back a test [`Vec<u8>`] after logging).
  pub fn writer_arc(&self) -> Arc<Mutex<W>> {
    self.writer.clone()
  }
}

#[derive(Serialize)]
struct JsonLine<'a> {
  level: &'a str,
  message: &'a str,
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  fields: HashMap<&'a str, &'a str>,
  #[serde(skip_serializing_if = "spans_is_empty")]
  spans: Vec<String>,
}

fn spans_is_empty(s: &[String]) -> bool {
  s.is_empty()
}

impl<W: Write + Send + 'static> LogBackend for JsonLogBackend<W> {
  fn emit(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError> {
    if rec.level == LogLevel::None {
      return Ok(());
    }
    let mut fields = HashMap::new();
    for (k, v) in rec.annotations.iter() {
      fields.insert(k.as_str(), v.as_str());
    }
    let row = JsonLine {
      level: rec.level.as_str(),
      message: rec.message.as_ref(),
      fields,
      spans: rec.spans.clone(),
    };
    let mut w = self
      .writer
      .lock()
      .map_err(|e| EffectLoggerError::Sink(format!("json backend mutex: {e}")))?;
    serde_json::to_writer(&mut *w, &row).map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    w.write_all(b"\n")
      .map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    Ok(())
  }
}

/// Human-oriented `key=value` lines (no JSON), still machine-grep-friendly.
#[derive(Clone)]
pub struct StructuredLogBackend<W: Write + Send + 'static> {
  writer: Arc<Mutex<W>>,
}

impl<W: Write + Send + 'static> StructuredLogBackend<W> {
  /// Wrap `writer`; each emit writes a human-oriented `key=value` line.
  pub fn new(writer: W) -> Self {
    Self {
      writer: Arc::new(Mutex::new(writer)),
    }
  }

  /// Shared handle to the underlying writer (e.g. read a test buffer after logging).
  pub fn writer_arc(&self) -> Arc<Mutex<W>> {
    self.writer.clone()
  }
}

impl<W: Write + Send + 'static> LogBackend for StructuredLogBackend<W> {
  fn emit(&self, rec: &LogRecord<'_>) -> Result<(), EffectLoggerError> {
    if rec.level == LogLevel::None {
      return Ok(());
    }
    let mut w = self
      .writer
      .lock()
      .map_err(|e| EffectLoggerError::Sink(format!("structured backend mutex: {e}")))?;
    write!(
      w,
      "level={} message={:?}",
      rec.level.as_str(),
      rec.message.as_ref()
    )
    .map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    if !rec.spans.is_empty() {
      write!(w, " spans={:?}", rec.spans.join(">"))
        .map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    }
    for (k, v) in rec.annotations.iter() {
      write!(w, " {k}={v:?}").map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    }
    w.write_all(b"\n")
      .map_err(|e| EffectLoggerError::Sink(e.to_string()))?;
    Ok(())
  }
}
