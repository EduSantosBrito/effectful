//! Method routers that wrap `Effect<A, E, S>` with [`axum::extract::State`].
//!
//! Each helper returns a [`MethodRouter`] you can pass to
//! [`axum::Router::route`].
//!
//! [`get_with_metrics`] (and siblings) increment a request [`Metric`]
//! counter and record handler wall time in a latency [`Metric`] (histogram / summary / timer) via
//! [`effect::Metric::track_duration`].

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{MethodFilter, MethodRouter};
use effect::Effect;
use effect::Metric;
use effect::duration::Duration;
use effect::runtime::run_blocking;

async fn run_with_axum_metrics<S, A, E, F>(
  env: S,
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> Result<A, E>
where
  S: Send + 'static,
  A: 'static,
  E: 'static,
  F: FnOnce(&mut S) -> Effect<A, E, S>,
{
  tokio::task::block_in_place(|| {
    run_blocking(request_counter.apply(1), ()).expect("request counter");
  });
  crate::run_effect_from_state(env, |e| latency.track_duration(f(e))).await
}

/// `GET` — `f` is invoked per request; use [`Clone`] on `f` when the router stores it (e.g. closure
/// with `Arc` captures).
#[inline]
pub fn get<S, A, E, F>(f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::get(move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `GET` with per-route request counting and latency recording.
///
/// Pass counters/histograms (typically tagged with `route`, etc.) built via [`Metric::counter`] /
/// [`Metric::histogram`].
#[inline]
pub fn get_with_metrics<S, A, E, F>(
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::get(move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `POST`
#[inline]
pub fn post<S, A, E, F>(f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::post(move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `POST` with request counter + latency [`Metric`] (see [`get_with_metrics`]).
#[inline]
pub fn post_with_metrics<S, A, E, F>(
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::post(move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `PUT`
#[inline]
pub fn put<S, A, E, F>(f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::put(move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `PUT` with request counter + latency [`Metric`].
#[inline]
pub fn put_with_metrics<S, A, E, F>(
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::put(move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `PATCH`
#[inline]
pub fn patch<S, A, E, F>(f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::patch(move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `PATCH` with request counter + latency [`Metric`].
#[inline]
pub fn patch_with_metrics<S, A, E, F>(
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::patch(move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `DELETE`
#[inline]
pub fn delete<S, A, E, F>(f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::delete(move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// `DELETE` with request counter + latency [`Metric`].
#[inline]
pub fn delete_with_metrics<S, A, E, F>(
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::delete(move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// Custom method filter (e.g. [`MethodFilter::HEAD`](MethodFilter::HEAD)).
#[inline]
pub fn on<S, A, E, F>(method: MethodFilter, f: F) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::on(method, move |st: State<S>| {
    let f = f.clone();
    async move {
      let State(env) = st;
      match crate::run_effect_from_state(env, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}

/// Custom method filter with request counter + latency [`Metric`].
#[inline]
pub fn on_with_metrics<S, A, E, F>(
  method: MethodFilter,
  request_counter: Metric<u64, ()>,
  latency: Metric<Duration, ()>,
  f: F,
) -> MethodRouter<S>
where
  S: Clone + Send + Sync + 'static,
  A: IntoResponse + Send + 'static,
  E: IntoResponse + Send + 'static,
  F: Fn(&mut S) -> Effect<A, E, S> + Clone + Send + Sync + 'static,
{
  axum::routing::on(method, move |st: State<S>| {
    let f = f.clone();
    let ctr = request_counter.clone();
    let lat = latency.clone();
    async move {
      let State(env) = st;
      match run_with_axum_metrics(env, ctr, lat, |e| f(e)).await {
        Ok(a) => a.into_response(),
        Err(e) => e.into_response(),
      }
    }
  })
}
