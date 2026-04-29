//! Method routers that wrap `Effect<A, E, S>` with [`axum::extract::State`].
//!
//! Each helper returns a [`MethodRouter`] you can pass to
//! [`axum::Router::route`].
//!
//! [`get_with_metrics`] (and siblings) increment a request [`Metric`]
//! counter and record handler wall time in a latency [`Metric`] (histogram / summary / timer) via
//! [`effectful::Metric::track_duration`].

use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{MethodFilter, MethodRouter};
use effectful::Effect;
use effectful::Metric;
use effectful::duration::Duration;
use effectful_tokio::EffectExecution;

/// Shared response helper: run the effect through [`EffectExecution`] and map `Result` to Axum
/// [`IntoResponse`].
async fn run_and_respond<S, A, E, F>(
  env: S,
  execution: EffectExecution,
  f: F,
) -> axum::response::Response
where
  S: Send + 'static,
  A: IntoResponse + 'static,
  E: IntoResponse + 'static,
  F: FnOnce(&mut S) -> Effect<A, E, S>,
{
  match effectful_tokio::run_effect_from_state_with(env, execution, f).await {
    Ok(a) => a.into_response(),
    Err(e) => e.into_response(),
  }
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
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
    async move { crate::execute(st, move |e| f(e)).await }
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
    let execution = EffectExecution::RouteMetrics {
      request_counter: request_counter.clone(),
      latency: latency.clone(),
    };
    async move {
      let State(env) = st;
      run_and_respond(env, execution, |e| f(e)).await
    }
  })
}

#[cfg(test)]
mod tests {
  use std::convert::Infallible;

  use axum::body::Body;
  use axum::http::{Method, Request, StatusCode};
  use axum::routing::{MethodFilter, Router};
  use effectful::duration::Duration;
  use effectful::{Effect, Metric, fail, succeed};
  use http_body_util::BodyExt;
  use tower::ServiceExt;

  use super::*;

  #[derive(Clone)]
  struct AppState(());

  fn ok(_: &mut AppState) -> Effect<&'static str, Infallible, AppState> {
    succeed("ok")
  }

  fn fail_handler(_: &mut AppState) -> Effect<(), (StatusCode, &'static str), AppState> {
    fail((StatusCode::INTERNAL_SERVER_ERROR, "nope"))
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn get_success_maps_effect_value_into_response() {
    let app = Router::new().route("/g", get(ok)).with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/g")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn get_error_maps_effect_error_into_response() {
    let app = Router::new()
      .route("/g", get(fail_handler))
      .with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::GET)
          .uri("/g")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"nope");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn post_success_maps_effect_value_into_response() {
    let app = Router::new().route("/p", post(ok)).with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/p")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn post_error_maps_effect_error_into_response() {
    let app = Router::new()
      .route("/p", post(fail_handler))
      .with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/p")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"nope");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn put_success_maps_effect_value_into_response() {
    let app = Router::new().route("/u", put(ok)).with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/u")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::OK);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"ok");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn put_error_maps_effect_error_into_response() {
    let app = Router::new()
      .route("/u", put(fail_handler))
      .with_state(AppState(()));

    let res = app
      .oneshot(
        Request::builder()
          .method(Method::PUT)
          .uri("/u")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"nope");
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn get_post_put_patch_delete_and_error_paths() {
    let app = Router::new()
      .route("/g", get(ok))
      .route("/p", post(ok))
      .route("/u", put(ok))
      .route("/a", patch(ok))
      .route("/d", delete(ok))
      .route("/e", get(fail_handler))
      .with_state(AppState(()));

    for (method, path) in [
      (Method::GET, "/g"),
      (Method::POST, "/p"),
      (Method::PUT, "/u"),
      (Method::PATCH, "/a"),
      (Method::DELETE, "/d"),
    ] {
      let res = app
        .clone()
        .oneshot(
          Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
      assert_eq!(res.status(), StatusCode::OK);
    }

    let res = app
      .oneshot(Request::builder().uri("/e").body(Body::empty()).unwrap())
      .await
      .unwrap();
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn on_and_metrics_variants_execute() {
    let ctr = Metric::counter("c", []);
    let lat = Metric::<Duration, ()>::histogram("h", []);
    let app = Router::new()
      .route("/gm", get_with_metrics(ctr.clone(), lat.clone(), ok))
      .route("/pm", post_with_metrics(ctr.clone(), lat.clone(), ok))
      .route("/um", put_with_metrics(ctr.clone(), lat.clone(), ok))
      .route("/am", patch_with_metrics(ctr.clone(), lat.clone(), ok))
      .route("/dm", delete_with_metrics(ctr.clone(), lat.clone(), ok))
      .route("/o", on(MethodFilter::OPTIONS, ok))
      .with_state(AppState(()));

    for (method, path) in [
      (Method::GET, "/gm"),
      (Method::POST, "/pm"),
      (Method::PUT, "/um"),
      (Method::PATCH, "/am"),
      (Method::DELETE, "/dm"),
    ] {
      let _ = app
        .clone()
        .oneshot(
          Request::builder()
            .method(method)
            .uri(path)
            .body(Body::empty())
            .unwrap(),
        )
        .await
        .unwrap();
    }

    let _ = app
      .clone()
      .oneshot(
        Request::builder()
          .method(Method::OPTIONS)
          .uri("/o")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    let app2 = Router::new()
      .route("/h", on_with_metrics(MethodFilter::HEAD, ctr, lat, ok))
      .with_state(AppState(()));
    let _ = app2
      .oneshot(
        Request::builder()
          .method(Method::HEAD)
          .uri("/h")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();
  }
}
