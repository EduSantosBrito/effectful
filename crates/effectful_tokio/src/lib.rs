//! Tokio integration for [`effectful`]: [`TokioRuntime`] implements [`effectful::Runtime`] with
//! cooperative sleep/yield, and **runs forked effects** on Tokio’s **blocking thread pool** via
//! [`tokio::runtime::Handle::spawn_blocking`] (the `Effect` interpreter is driven with
//! [`run_blocking`]; it is not `Send` for [`tokio::spawn`]).
//!
//! Tower, Axum, and other Tokio-based adapters should depend on **`effectful_tokio`** for this wiring.
//!
//! ## Examples
//!
//! See `examples/` (e.g. `109_tokio_end_to_end`). Re-exports
//! [`run_async`], [`run_blocking`], [`run_fork`], and [`yield_now`] from `effectful` for use at the
//! async boundary alongside [`TokioRuntime`].

#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use pin_project_lite::pin_project;

use effectful::{Effect, FiberHandle, FiberId, Metric, Never, Runtime, from_async};
use effectful::Duration as EffectfulDuration;

/// Commonly used at the async boundary together with [`TokioRuntime`].
pub use effectful::{run_async, run_blocking, run_fork, yield_now};

/// Execution mode for [`run_effect_from_state_with`].
///
/// Using an enum makes illegal metric states unrepresentable: a plain run has no metrics,
/// route metrics have request counter + latency, and service metrics have latency + errors.
#[derive(Clone)]
pub enum EffectExecution {
  /// No metrics; equivalent to [`run_effect_from_state`].
  Plain,
  /// Axum route metrics: increment a request counter and record handler latency.
  RouteMetrics {
    /// Request counter incremented once per call.
    request_counter: Metric<u64, ()>,
    /// Latency histogram / timer / summary.
    latency: Metric<EffectfulDuration, ()>,
  },
  /// Tower service metrics: record request latency and increment an error counter on failure.
  ServiceMetrics {
    /// Latency histogram / timer / summary.
    latency: Metric<EffectfulDuration, ()>,
    /// Error counter incremented when the effect returns `Err`.
    errors: Metric<u64, ()>,
  },
}

/// Run `build(&mut env)` to obtain an effect, then drive it to completion with the **same** `env`.
///
/// Uses [`tokio::task::block_in_place`] / [`tokio::runtime::Handle::block_on`]
/// so the returned future is [`Send`] for Axum / Tower HTTP handlers. Requires a **multi-thread**
/// Tokio runtime (the default for `#[tokio::main]`).
#[inline]
pub async fn run_effect_from_state_with<S, A, E, F>(
  mut env: S,
  execution: EffectExecution,
  build: F,
) -> Result<A, E>
where
  S: Send + 'static,
  A: 'static,
  E: 'static,
  F: FnOnce(&mut S) -> Effect<A, E, S>,
{
  tokio::task::block_in_place(move || {
    tokio::runtime::Handle::current().block_on(async move {
      match execution {
        EffectExecution::Plain => {
          let eff = build(&mut env);
          run_async(eff, env).await
        }
        EffectExecution::RouteMetrics {
          request_counter,
          latency,
        } => {
          let _ = run_async(request_counter.apply(1), ()).await;
          let eff = build(&mut env);
          let eff = latency.track_duration(eff);
          run_async(eff, env).await
        }
        EffectExecution::ServiceMetrics { latency, errors } => {
          let eff = build(&mut env);
          let eff = latency.track_duration(eff);
          let result = run_async(eff, env).await;
          if result.is_err() {
            let _ = run_async(errors.apply(1), ()).await;
          }
          result
        }
      }
    })
  })
}

/// Convenience wrapper: [`run_effect_from_state_with`] with [`EffectExecution::Plain`].
#[inline]
pub async fn run_effect_from_state<S, A, E, F>(env: S, build: F) -> Result<A, E>
where
  S: Send + 'static,
  A: 'static,
  E: 'static,
  F: FnOnce(&mut S) -> Effect<A, E, S>,
{
  run_effect_from_state_with(env, EffectExecution::Plain, build).await
}

/// Tokio-backed [`Runtime`] adapter (async `sleep` / `yield_now`).
pub struct TokioRuntime {
  _owned: Option<Arc<tokio::runtime::Runtime>>,
  _handle: tokio::runtime::Handle,
}

impl TokioRuntime {
  /// Adapter for the current Tokio context.
  pub fn current() -> Result<Self, std::io::Error> {
    let handle = tokio::runtime::Handle::try_current()
      .map_err(|e| std::io::Error::other(format!("no current tokio runtime: {e}")))?;
    Ok(Self {
      _owned: None,
      _handle: handle,
    })
  }

  /// Adapter from an explicit Tokio handle (e.g. `axum::serve` / `#[tokio::main]`).
  #[inline]
  pub fn from_handle(handle: tokio::runtime::Handle) -> Self {
    Self {
      _owned: None,
      _handle: handle,
    }
  }

  /// Owns a single-threaded Tokio runtime (tests, examples, `main` without `#[tokio::main]`).
  pub fn new_current_thread() -> Result<Self, std::io::Error> {
    let runtime = tokio::runtime::Builder::new_current_thread()
      .enable_time()
      .build()?;
    let runtime = Arc::new(runtime);
    let handle = runtime.handle().clone();
    Ok(Self {
      _owned: Some(runtime),
      _handle: handle,
    })
  }

  /// Owns a multi-thread Tokio runtime (typical for CLIs and servers without `#[tokio::main]`).
  pub fn new_multi_thread() -> Result<Self, std::io::Error> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
      .enable_all()
      .build()?;
    let runtime = Arc::new(runtime);
    let handle = runtime.handle().clone();
    Ok(Self {
      _owned: Some(runtime),
      _handle: handle,
    })
  }

  /// Tokio handle for this adapter (same underlying runtime as [`Self::block_on`] when owned).
  #[inline]
  pub fn handle(&self) -> tokio::runtime::Handle {
    self._handle.clone()
  }

  /// Run a future on the owned runtime when this adapter was built with [`Self::new_current_thread`]
  /// or [`Self::new_multi_thread`].
  ///
  /// When constructed with [`Self::from_handle`] / [`Self::current`], this panics — use the
  /// surrounding runtime’s `block_on` instead.
  pub fn block_on<F: std::future::Future>(&self, f: F) -> F::Output {
    match &self._owned {
      Some(rt) => rt.block_on(f),
      None => panic!(
        "TokioRuntime::block_on requires TokioRuntime::new_current_thread() or new_multi_thread(); \
         otherwise use your Runtime::block_on / #[tokio::main] with from_handle"
      ),
    }
  }
}

impl Runtime for TokioRuntime {
  fn spawn_with<A, E, R, F>(&self, f: F) -> FiberHandle<A, E>
  where
    A: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
    R: 'static,
    F: FnOnce() -> (Effect<A, E, R>, R) + Send + 'static,
  {
    let handle = FiberHandle::pending(FiberId::fresh());
    let h = handle.clone();
    let rt = self._handle.clone();
    // `run_async` is not `Send`; drive the effect with `run_blocking` on Tokio's blocking pool.
    let _join = rt.spawn_blocking(move || {
      let (effect, env) = f();
      h.mark_completed(run_blocking(effect, env));
    });
    handle
  }

  #[inline(always)]
  fn sleep(&self, duration: Duration) -> Effect<(), Never, ()> {
    from_async(move |_env| async move {
      tokio::time::sleep(duration).await;
      Ok::<(), Never>(())
    })
  }

  #[inline]
  fn now(&self) -> Instant {
    instant_now_blocking()
  }

  #[inline(always)]
  fn yield_now(&self) -> Effect<(), Never, ()> {
    Effect::new_static_async_fn(tokio_yield_now_effect)
  }
}

impl TokioRuntime {
  /// Fast-path yield that bypasses `BoxFuture` allocation.
  #[inline(always)]
  pub fn yield_now_fast(&self) -> YieldNow {
    YieldNow
  }
}

pin_project! {
  struct YieldNowEffect<F> {
    #[pin]
    inner: F,
  }
}

impl<F> YieldNowEffect<F>
where
  F: Future<Output = ()>,
{
  #[inline(always)]
  fn new(inner: F) -> Self {
    Self { inner }
  }
}

impl<F> Future for YieldNowEffect<F>
where
  F: Future<Output = ()>,
{
  type Output = Result<(), Never>;

  #[inline(always)]
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    self.project().inner.poll(cx).map(|_| Ok(()))
  }
}

/// Zero-allocation yield token for use in `effect!` macros.
pub struct YieldNow;

impl<'a> effectful::IntoBindFastExt<'a, (), (), Never> for YieldNow {
  fn __into_bind_fast(self, _r: &'a mut ()) -> impl Future<Output = Result<(), Never>> + 'a {
    YieldNowEffect::new(tokio::task::yield_now())
  }
}

fn tokio_yield_now_effect(_env: &mut ()) -> effectful::BoxFuture<'static, Result<(), Never>> {
  effectful::box_future(YieldNowEffect::new(tokio::task::yield_now()))
}

#[inline]
fn instant_now_blocking() -> Instant {
  // Dylint `effect_no_instant_now_outside_boundary`: wall time is only allowed in `run_*`-adjacent
  // helpers (`*_blocking`); `Runtime::now` is the Tokio clock boundary for `LiveClock` / scheduling.
  Instant::now()
}

#[cfg(test)]
mod tests {
  use super::*;
  use effectful::kernel::{fail, succeed};
  use std::time::Duration;

  #[test]
  fn new_current_thread_runs_sleep_and_yield_under_block_on() {
    let rt = TokioRuntime::new_current_thread().expect("tokio runtime should build");
    rt.block_on(async {
      assert_eq!(
        run_async(rt.sleep(Duration::from_millis(0)), ()).await,
        Ok(())
      );
      assert_eq!(run_async(yield_now(&rt), ()).await, Ok(()));
    });
  }

  #[test]
  fn spawn_runs_effect_to_completion_on_runtime() {
    let rt = TokioRuntime::new_current_thread().expect("tokio runtime should build");
    rt.block_on(async {
      let h = run_fork(&rt, || (succeed::<u8, (), ()>(7), ()));
      assert_eq!(h.join().await, Ok(7));
    });
  }

  #[tokio::test]
  async fn from_handle_uses_current_context() {
    let handle = tokio::runtime::Handle::current();
    let rt = TokioRuntime::from_handle(handle);
    // sleep and yield_now work under current context
    assert_eq!(
      run_async(rt.sleep(Duration::from_millis(0)), ()).await,
      Ok(())
    );
    assert_eq!(run_async(yield_now(&rt), ()).await, Ok(()));
  }

  #[tokio::test]
  async fn current_succeeds_inside_tokio_context() {
    let rt = TokioRuntime::current().expect("current should work inside #[tokio::test]");
    assert_eq!(
      run_async(rt.sleep(Duration::from_millis(0)), ()).await,
      Ok(())
    );
  }

  #[test]
  fn now_returns_monotonic_instant() {
    let rt = TokioRuntime::new_current_thread().expect("runtime");
    let t1 = rt.now();
    let t2 = rt.now();
    assert!(t2 >= t1, "now() should be non-decreasing");
  }

  #[test]
  fn new_multi_thread_block_on_runs_async() {
    let rt = TokioRuntime::new_multi_thread().expect("multi-thread runtime should build");
    rt.block_on(async {
      assert_eq!(
        run_async(rt.sleep(Duration::from_millis(0)), ()).await,
        Ok(())
      );
    });
  }

  #[test]
  fn current_fails_when_no_tokio_runtime() {
    let res = std::thread::spawn(TokioRuntime::current)
      .join()
      .expect("thread should not panic");
    let err = match res {
      Err(e) => e,
      Ok(_) => panic!("expected Err outside a Tokio context"),
    };
    assert!(
      err.to_string().contains("no current tokio runtime"),
      "unexpected error: {err}"
    );
  }

  #[test]
  #[should_panic(expected = "TokioRuntime::block_on requires")]
  fn block_on_panics_when_adapter_has_no_owned_runtime() {
    let owned = TokioRuntime::new_current_thread().expect("runtime");
    let adapter = TokioRuntime::from_handle(owned.handle());
    adapter.block_on(async {});
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn run_effect_from_state_with_plain_runs_effect() {
    let result = run_effect_from_state_with((), EffectExecution::Plain, |_e| {
      succeed::<u32, (), ()>(42)
    })
    .await;
    assert_eq!(result, Ok(42));
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn run_effect_from_state_with_route_metrics_increments_counter_and_records_latency() {
    let ctr = Metric::counter("req", std::iter::empty());
    let lat = Metric::<EffectfulDuration, ()>::histogram("lat", std::iter::empty());
    let result = run_effect_from_state_with(
      (),
      EffectExecution::RouteMetrics {
        request_counter: ctr.clone(),
        latency: lat.clone(),
      },
      |_e| succeed::<u32, (), ()>(99),
    )
    .await;
    assert_eq!(result, Ok(99));
    assert_eq!(ctr.snapshot_count(), 1);
    assert_eq!(lat.snapshot_durations().len(), 1);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn run_effect_from_state_with_service_metrics_records_latency_and_increments_errors() {
    let lat = Metric::<EffectfulDuration, ()>::histogram("lat", std::iter::empty());
    let err = Metric::counter("err", std::iter::empty());
    let result = run_effect_from_state_with(
      (),
      EffectExecution::ServiceMetrics {
        latency: lat.clone(),
        errors: err.clone(),
      },
      |_e| fail::<u32, &str, ()>("boom"),
    )
    .await;
    assert_eq!(result, Err("boom"));
    assert_eq!(err.snapshot_count(), 1);
    assert_eq!(lat.snapshot_durations().len(), 1);
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
  async fn run_effect_from_state_with_service_metrics_does_not_increment_errors_on_success() {
    let lat = Metric::<EffectfulDuration, ()>::histogram("lat2", std::iter::empty());
    let err = Metric::counter("err2", std::iter::empty());
    let result = run_effect_from_state_with(
      (),
      EffectExecution::ServiceMetrics {
        latency: lat.clone(),
        errors: err.clone(),
      },
      |_e| succeed::<u32, &str, ()>(7),
    )
    .await;
    assert_eq!(result, Ok(7));
    assert_eq!(err.snapshot_count(), 0);
    assert_eq!(lat.snapshot_durations().len(), 1);
  }
}
