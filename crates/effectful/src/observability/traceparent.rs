//! W3C `traceparent` header parsing and rendering.
//!
//! This module is intentionally separate from span collection and tracing
//! backends so that traceparent parsing can be tested and used without
//! configuring any tracing infrastructure.

use std::fmt;

/// OpenTelemetry-shaped trace identifier: 16 opaque bytes rendered as 32 lowercase hex chars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TraceId([u8; 16]);

impl TraceId {
  /// Builds a trace id from its OTel byte representation.
  #[inline]
  pub const fn from_bytes(bytes: [u8; 16]) -> Self {
    Self(bytes)
  }

  /// Returns the OTel byte representation.
  #[inline]
  pub const fn to_bytes(self) -> [u8; 16] {
    self.0
  }
}

impl fmt::Display for TraceId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt_hex(&self.0, f)
  }
}

/// OpenTelemetry-shaped span identifier: 8 opaque bytes rendered as 16 lowercase hex chars.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId([u8; 8]);

impl SpanId {
  /// Builds a span id from its OTel byte representation.
  #[inline]
  pub const fn from_bytes(bytes: [u8; 8]) -> Self {
    Self(bytes)
  }

  /// Returns the OTel byte representation.
  #[inline]
  pub const fn to_bytes(self) -> [u8; 8] {
    self.0
  }
}

impl fmt::Display for SpanId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt_hex(&self.0, f)
  }
}

/// W3C/OTel trace flags. Bit `0x01` is the sampled flag.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct TraceFlags {
  bits: u8,
}

impl TraceFlags {
  /// No trace flags set.
  pub const DEFAULT: Self = Self { bits: 0 };
  /// Sampled trace flag set.
  pub const SAMPLED: Self = Self { bits: 1 };

  /// Builds flags from raw W3C/OTel bits.
  #[inline]
  pub const fn from_bits(bits: u8) -> Self {
    Self { bits }
  }

  /// Returns raw W3C/OTel bits.
  #[inline]
  pub const fn bits(self) -> u8 {
    self.bits
  }

  /// Returns true when the sampled flag is set.
  #[inline]
  pub const fn sampled(self) -> bool {
    self.bits & Self::SAMPLED.bits == Self::SAMPLED.bits
  }
}

impl fmt::Display for TraceFlags {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:02x}", self.bits)
  }
}

/// Errors returned by W3C `traceparent` parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceParentParseError {
  /// Header did not have the four required dash-separated fields.
  InvalidFieldCount,
  /// Only version `00` is supported.
  UnsupportedVersion,
  /// A hex field had the wrong encoded length.
  InvalidLength,
  /// A hex field contained a non-hex character.
  InvalidHex,
  /// W3C forbids all-zero trace ids and span ids.
  AllZeroId,
}

/// Trace context attached to every recorded span.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanContext {
  /// Trace id shared by all spans in one distributed trace.
  pub trace_id: TraceId,
  /// Span id unique within a trace.
  pub span_id: SpanId,
  /// W3C/OTel flags for the trace.
  pub trace_flags: TraceFlags,
}

impl SpanContext {
  /// Renders this context as a W3C `traceparent` header value.
  #[inline]
  pub fn to_traceparent(self) -> String {
    format!("00-{}-{}-{}", self.trace_id, self.span_id, self.trace_flags)
  }

  /// Parses a W3C `traceparent` header value into a span context.
  pub fn from_traceparent(value: &str) -> Result<Self, TraceParentParseError> {
    let mut parts = value.split('-');
    let version = parts
      .next()
      .ok_or(TraceParentParseError::InvalidFieldCount)?;
    let trace_id = parts
      .next()
      .ok_or(TraceParentParseError::InvalidFieldCount)?;
    let span_id = parts
      .next()
      .ok_or(TraceParentParseError::InvalidFieldCount)?;
    let trace_flags = parts
      .next()
      .ok_or(TraceParentParseError::InvalidFieldCount)?;
    if parts.next().is_some() {
      return Err(TraceParentParseError::InvalidFieldCount);
    }
    if version != "00" {
      return Err(TraceParentParseError::UnsupportedVersion);
    }

    let trace_id = TraceId::from_bytes(parse_hex_array::<16>(trace_id)?);
    let span_id = SpanId::from_bytes(parse_hex_array::<8>(span_id)?);
    if trace_id.to_bytes().iter().all(|byte| *byte == 0)
      || span_id.to_bytes().iter().all(|byte| *byte == 0)
    {
      return Err(TraceParentParseError::AllZeroId);
    }
    let [trace_flags] = parse_hex_array::<1>(trace_flags)?;

    Ok(Self {
      trace_id,
      span_id,
      trace_flags: TraceFlags::from_bits(trace_flags),
    })
  }
}

fn fmt_hex(bytes: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result {
  for byte in bytes {
    write!(f, "{byte:02x}")?;
  }
  Ok(())
}

fn parse_hex_array<const N: usize>(value: &str) -> Result<[u8; N], TraceParentParseError> {
  if value.len() != N * 2 {
    return Err(TraceParentParseError::InvalidLength);
  }
  let mut out = [0; N];
  let bytes = value.as_bytes();
  for index in 0..N {
    let high = hex_nibble(bytes[index * 2])?;
    let low = hex_nibble(bytes[index * 2 + 1])?;
    out[index] = (high << 4) | low;
  }
  Ok(out)
}

fn hex_nibble(byte: u8) -> Result<u8, TraceParentParseError> {
  match byte {
    b'0'..=b'9' => Ok(byte - b'0'),
    b'a'..=b'f' => Ok(byte - b'a' + 10),
    b'A'..=b'F' => Ok(byte - b'A' + 10),
    _ => Err(TraceParentParseError::InvalidHex),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use rstest::rstest;

  // ---------- SpanContext::to_traceparent / from_traceparent ----------

  #[test]
  fn span_context_renders_and_parses_traceparent() {
    let context = SpanContext {
      trace_id: TraceId::from_bytes([1; 16]),
      span_id: SpanId::from_bytes([2; 8]),
      trace_flags: TraceFlags::SAMPLED,
    };
    let header = context.to_traceparent();
    assert_eq!(
      header,
      "00-01010101010101010101010101010101-0202020202020202-01"
    );
    assert_eq!(SpanContext::from_traceparent(&header), Ok(context));
  }

  #[rstest]
  #[case::all_zero_trace_id(
    "00-00000000000000000000000000000000-0202020202020202-01",
    TraceParentParseError::AllZeroId
  )]
  #[case::all_zero_span_id(
    "00-01010101010101010101010101010101-0000000000000000-01",
    TraceParentParseError::AllZeroId
  )]
  #[case::too_many_fields(
    "00-01010101010101010101010101010101-0202020202020202-01-extra",
    TraceParentParseError::InvalidFieldCount
  )]
  #[case::too_few_fields(
    "00-01010101010101010101010101010101-0202020202020202",
    TraceParentParseError::InvalidFieldCount
  )]
  #[case::unsupported_version(
    "01-01010101010101010101010101010101-0202020202020202-01",
    TraceParentParseError::UnsupportedVersion
  )]
  #[case::invalid_trace_id_length(
    "00-0101010101010101010101010101010-0202020202020202-01",
    TraceParentParseError::InvalidLength
  )]
  #[case::invalid_span_id_length(
    "00-01010101010101010101010101010101-020202020202020-01",
    TraceParentParseError::InvalidLength
  )]
  #[case::invalid_flags_length(
    "00-01010101010101010101010101010101-0202020202020202-1",
    TraceParentParseError::InvalidLength
  )]
  #[case::invalid_hex_in_trace_id(
    "00-0101010101010101010101010101010g-0202020202020202-01",
    TraceParentParseError::InvalidHex
  )]
  #[case::invalid_hex_in_span_id(
    "00-01010101010101010101010101010101-020202020202020g-01",
    TraceParentParseError::InvalidHex
  )]
  #[case::invalid_hex_in_flags(
    "00-01010101010101010101010101010101-0202020202020202-0g",
    TraceParentParseError::InvalidHex
  )]
  fn from_traceparent_rejects_malformed_input(
    #[case] input: &str,
    #[case] expected: TraceParentParseError,
  ) {
    assert_eq!(SpanContext::from_traceparent(input), Err(expected));
  }

  #[test]
  fn from_traceparent_accepts_uppercase_hex() {
    let context = SpanContext {
      trace_id: TraceId::from_bytes([0xAB; 16]),
      span_id: SpanId::from_bytes([0xCD; 8]),
      trace_flags: TraceFlags::SAMPLED,
    };
    assert_eq!(
      SpanContext::from_traceparent("00-ABABABABABABABABABABABABABABABAB-CDCDCDCDCDCDCDCD-01"),
      Ok(context)
    );
  }

  #[test]
  fn from_traceparent_parses_unsampled_flag() {
    let context = SpanContext {
      trace_id: TraceId::from_bytes([1; 16]),
      span_id: SpanId::from_bytes([2; 8]),
      trace_flags: TraceFlags::DEFAULT,
    };
    assert_eq!(
      SpanContext::from_traceparent("00-01010101010101010101010101010101-0202020202020202-00"),
      Ok(context)
    );
  }

  // ---------- TraceFlags ----------

  #[test]
  fn trace_flags_default_is_not_sampled() {
    assert!(!TraceFlags::DEFAULT.sampled());
    assert_eq!(TraceFlags::DEFAULT.bits(), 0);
  }

  #[test]
  fn trace_flags_sampled_is_sampled() {
    assert!(TraceFlags::SAMPLED.sampled());
    assert_eq!(TraceFlags::SAMPLED.bits(), 1);
  }

  #[test]
  fn trace_flags_from_bits_roundtrips() {
    let flags = TraceFlags::from_bits(0x03);
    assert_eq!(flags.bits(), 0x03);
    assert!(flags.sampled());
  }

  // ---------- Display ----------

  #[test]
  fn trace_id_display_renders_lowercase_hex() {
    assert_eq!(
      TraceId::from_bytes([0xAB; 16]).to_string(),
      "abababababababababababababababab"
    );
  }

  #[test]
  fn span_id_display_renders_lowercase_hex() {
    assert_eq!(
      SpanId::from_bytes([0xCD; 8]).to_string(),
      "cdcdcdcdcdcdcdcd"
    );
  }

  #[test]
  fn trace_flags_display_renders_two_digit_hex() {
    assert_eq!(TraceFlags::from_bits(0x0F).to_string(), "0f");
    assert_eq!(TraceFlags::from_bits(0xFF).to_string(), "ff");
  }
}
