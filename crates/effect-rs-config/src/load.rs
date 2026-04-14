//! Low-level reads against the injected [`crate::ConfigProvider`] service.
//!
//! Every public function returns `Effect<A, E, R>` where `R: NeedsConfigProvider`.
//! The provider is extracted synchronously from the environment via
//! `Get::<ConfigProviderKey, Here>::get(r)` so all effects stay non-async and
//! the `EFFECT_PREFER_FROM_ASYNC_OVER_NEW_ASYNC` lint is never triggered.

use ::effect::{Effect, Get, Here, effect};

use crate::error::ConfigError;
use crate::provider::{ConfigProviderKey, NeedsConfigProvider};

// ── helpers ──────────────────────────────────────────────────────────────────

/// `Config.withDefault` as a method — only [`ConfigError::Missing`] is swapped.
///
/// This is a trait-impl method so the `effect!` lint is not required here.
pub trait WithConfigDefault<A, R>: Sized {
  /// Return `default` in place of the effect value when the error is [`ConfigError::Missing`].
  fn with_default(self, default: A) -> Effect<A, ConfigError, R>;
}

impl<A, R> WithConfigDefault<A, R> for Effect<A, ConfigError, R>
where
  A: Clone + 'static,
  R: 'static,
{
  fn with_default(self, default: A) -> Effect<A, ConfigError, R> {
    self.catch(move |e| match e {
      ConfigError::Missing { .. } => ::effect::succeed(default.clone()),
      other => ::effect::fail(other),
    })
  }
}

// ── path helpers ──────────────────────────────────────────────────────────────

/// Build a multi-segment path, e.g. `nested_path("SERVER", &["HOST"])` → `["SERVER", "HOST"]`.
#[inline]
pub fn nested_path(namespace: &str, leaf: &[&str]) -> Vec<String> {
  std::iter::once(namespace.to_string())
    .chain(leaf.iter().map(|s| (*s).to_string()))
    .collect()
}

// ── primitive reads ───────────────────────────────────────────────────────────

/// Required string.
pub fn read_string<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<String> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => return Err(E::from(ConfigError::Missing { path: path_str })),
      Ok(Some(s)) => A::from(s),
    }
  })
}

/// Optional string — missing key yields `None`.
pub fn read_string_opt<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<Option<String>> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(raw) => A::from(raw),
    }
  })
}

/// Floating-point number parsed from a string scalar.
pub fn read_number<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<f64> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    let s = match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => {
        return Err(E::from(ConfigError::Missing {
          path: path_str.clone(),
        }));
      }
      Ok(Some(s)) => s,
    };
    let n = s.parse::<f64>().map_err(|e| {
      E::from(ConfigError::Invalid {
        path: path_str,
        value: s,
        reason: e.to_string(),
      })
    })?;
    A::from(n)
  })
}

/// Signed 64-bit integer parsed from a string scalar.
pub fn read_i64<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<i64> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    let s = match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => {
        return Err(E::from(ConfigError::Missing {
          path: path_str.clone(),
        }));
      }
      Ok(Some(s)) => s,
    };
    let n = s.parse::<i64>().map_err(|e| {
      E::from(ConfigError::Invalid {
        path: path_str,
        value: s,
        reason: e.to_string(),
      })
    })?;
    A::from(n)
  })
}

/// Boolean parsed from `"true"` / `"false"` / `"1"` / `"0"` / `"yes"` / `"no"`.
pub fn read_bool<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<bool> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    let s = match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => {
        return Err(E::from(ConfigError::Missing {
          path: path_str.clone(),
        }));
      }
      Ok(Some(s)) => s,
    };
    let b = match s.to_ascii_lowercase().as_str() {
      "true" | "1" | "yes" => true,
      "false" | "0" | "no" => false,
      _ => {
        return Err(E::from(ConfigError::Invalid {
          path: path_str,
          value: s,
          reason: "expected boolean string".into(),
        }));
      }
    };
    A::from(b)
  })
}

/// Sequence of strings split by [`ConfigProvider::seq_delim`].
pub fn read_string_list<A, E, R>(path: &[&str]) -> Effect<A, E, R>
where
  A: From<Vec<String>> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned: Vec<String> = path.iter().map(|s| s.to_string()).collect();
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    let s = match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => return Err(E::from(ConfigError::Missing { path: path_str })),
      Ok(Some(s)) => s,
    };
    let delim = provider.0.seq_delim();
    let list: Vec<String> = s
      .split(delim)
      .map(str::trim)
      .filter(|x| !x.is_empty())
      .map(str::to_string)
      .collect();
    A::from(list)
  })
}

// ── nested convenience ────────────────────────────────────────────────────────

/// [`nested_path`] then [`read_string`].
pub fn read_nested_string<A, E, R>(namespace: &str, leaf: &[&str]) -> Effect<A, E, R>
where
  A: From<String> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned = nested_path(namespace, leaf);
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => return Err(E::from(ConfigError::Missing { path: path_str })),
      Ok(Some(s)) => A::from(s),
    }
  })
}

/// [`nested_path`] then [`read_string_list`].
pub fn read_nested_string_list<A, E, R>(namespace: &str, leaf: &[&str]) -> Effect<A, E, R>
where
  A: From<Vec<String>> + 'static,
  E: From<ConfigError> + 'static,
  R: NeedsConfigProvider + 'static,
{
  let path_owned = nested_path(namespace, leaf);
  effect!(|r: &mut R| {
    let provider = Get::<ConfigProviderKey, Here>::get(r);
    let refs: Vec<&str> = path_owned.iter().map(String::as_str).collect();
    let path_str = refs.join(".");
    let s = match provider.0.load_raw(&refs) {
      Err(e) => return Err(E::from(e)),
      Ok(None) => return Err(E::from(ConfigError::Missing { path: path_str })),
      Ok(Some(s)) => s,
    };
    let delim = provider.0.seq_delim();
    let list: Vec<String> = s
      .split(delim)
      .map(str::trim)
      .filter(|x| !x.is_empty())
      .map(str::to_string)
      .collect();
    A::from(list)
  })
}
