//! Deterministic test runtime harness helpers.
//!
//! Prefer effect-returning tests over calling [`Effect::run`](crate::Effect::run) in test bodies.
//! The adapter owns execution, environment creation, hygiene checks, and failure formatting.
//!
//! ```rust
//! use effectful::{Effect, effect_test};
//!
//! #[effect_test]
//! fn succeeds() -> Effect<(), &'static str, ()> {
//!   Effect::new(|_| Ok(()))
//! }
//! ```
//!
//! `#[effect_test]` expands to a current-thread Tokio test through `effectful::testing`, so
//! downstream crates do not need to name `tokio` in each test body.
//!
//! Provide a service context fixture with `env = "path"`:
//!
//! ```rust
//! use effectful::{Effect, Service, ServiceContext, effect_test};
//!
//! #[derive(Clone, Service)]
//! struct Clock { value: u64 }
//!
//! fn test_env() -> ServiceContext {
//!   Clock { value: 1 }.to_context()
//! }
//!
//! #[effect_test(env = "test_env")]
//! fn uses_context() -> Effect<(), effectful::MissingService, ServiceContext> {
//!   Effect::service::<Clock>().map(|clock| assert_eq!(clock.value, 1))
//! }
//! ```

use crate::layer::Layer;
use crate::runtime::Never;
use crate::{Effect, Exit, ServiceContext, TestClock};
use std::cell::{Cell, RefCell};
use std::fmt::Debug;

thread_local! {
  static LEAKED_FIBERS: Cell<usize> = const { Cell::new(0) };
  static UNCLOSED_SCOPES: Cell<usize> = const { Cell::new(0) };
  static ACTIVE_TEST_CLOCK: RefCell<Option<TestClock>> = const { RefCell::new(None) };
}

struct TestClockScope {
  previous: Option<TestClock>,
}

impl Drop for TestClockScope {
  fn drop(&mut self) {
    let previous = self.previous.clone();
    ACTIVE_TEST_CLOCK.with(|clock| {
      *clock.borrow_mut() = previous;
    });
  }
}

fn install_test_clock(clock: TestClock) -> TestClockScope {
  let previous = ACTIVE_TEST_CLOCK.with(|active| active.borrow_mut().replace(clock));
  TestClockScope { previous }
}

pub(crate) fn current_test_clock() -> Option<TestClock> {
  ACTIVE_TEST_CLOCK.with(|clock| clock.borrow().clone())
}

fn reset_counters() {
  LEAKED_FIBERS.with(|c| c.set(0));
  UNCLOSED_SCOPES.with(|c| c.set(0));
}

fn assert_hygiene_counters() {
  let fiber_leaks = LEAKED_FIBERS.with(|c| c.get());
  assert_eq!(
    fiber_leaks, 0,
    "deterministic test harness detected leaked fibers: {fiber_leaks}"
  );

  let scope_leaks = UNCLOSED_SCOPES.with(|c| c.get());
  assert_eq!(
    scope_leaks, 0,
    "deterministic test harness detected unclosed scopes: {scope_leaks}"
  );
}

/// Internal hook for tests that need to simulate leaked fibers.
pub fn record_leaked_fiber() {
  LEAKED_FIBERS.with(|c| c.set(c.get().saturating_add(1)));
}

/// Internal hook for tests that need to simulate unclosed scopes.
pub fn record_unclosed_scope() {
  UNCLOSED_SCOPES.with(|c| c.set(c.get().saturating_add(1)));
}

/// Assert that no leaked fibers were recorded in the current test harness run.
pub fn assert_no_leaked_fibers() -> Effect<(), Never, ()> {
  Effect::new(move |_env| {
    let leaks = LEAKED_FIBERS.with(|c| c.get());
    assert_eq!(
      leaks, 0,
      "deterministic test harness detected leaked fibers: {leaks}"
    );
    Ok(())
  })
}

/// Assert that no unclosed scopes were recorded in the current test harness run.
pub fn assert_no_unclosed_scopes() -> Effect<(), Never, ()> {
  Effect::new(move |_env| {
    let leaks = UNCLOSED_SCOPES.with(|c| c.get());
    assert_eq!(
      leaks, 0,
      "deterministic test harness detected unclosed scopes: {leaks}"
    );
    Ok(())
  })
}

/// Small async runtime adapter for effect-returning tests.
///
/// `TestRuntime` lets downstream crates keep direct effect execution at the test harness edge.
/// Use [`TestRuntime::default`] for `R: Default`, or [`TestRuntime::with_env`] to provide a
/// context/fixture per test run.
pub struct TestRuntime<R, F = fn() -> R>
where
  R: 'static,
  F: FnOnce() -> R,
{
  make_env: F,
}

impl<R> Default for TestRuntime<R>
where
  R: Default + 'static,
{
  fn default() -> Self {
    Self {
      make_env: R::default,
    }
  }
}

impl<R, F> TestRuntime<R, F>
where
  R: 'static,
  F: FnOnce() -> R,
{
  /// Create a test runtime from an environment fixture.
  #[inline]
  pub fn with_env(make_env: F) -> Self {
    Self { make_env }
  }

  /// Run an effect under the test harness and return its result.
  #[inline]
  pub async fn run<A, E>(self, effect: Effect<A, E, R>) -> Result<A, E>
  where
    A: 'static,
    E: 'static,
  {
    let env = (self.make_env)();
    run_effect_test_with_env(effect, env).await
  }

  /// Run an effect under the test harness and panic with `Debug` output on failure.
  #[inline]
  pub async fn expect<A, E>(self, effect: Effect<A, E, R>) -> A
  where
    A: 'static,
    E: Debug + 'static,
  {
    expect_effect_test_with_env(effect, (self.make_env)()).await
  }
}

/// Run an effect-returning test with a default environment.
#[inline]
pub async fn run_effect_test<A, E, R>(effect: Effect<A, E, R>) -> Result<A, E>
where
  A: 'static,
  E: 'static,
  R: Default + 'static,
{
  run_effect_test_with_env(effect, R::default()).await
}

/// Run an effect-returning test with a provided environment.
#[inline]
pub async fn run_effect_test_with_env<A, E, R>(effect: Effect<A, E, R>, mut env: R) -> Result<A, E>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  reset_counters();
  let result = effect.run(&mut env).await;
  assert_hygiene_counters();
  result
}

/// Run an effect-returning test with a default environment and panic on failure.
#[inline]
pub async fn expect_effect_test<A, E, R>(effect: Effect<A, E, R>) -> A
where
  A: 'static,
  E: Debug + 'static,
  R: Default + 'static,
{
  expect_effect_test_with_env(effect, R::default()).await
}

/// Run an effect-returning test with a provided environment and panic on failure.
#[inline]
pub async fn expect_effect_test_with_env<A, E, R>(effect: Effect<A, E, R>, env: R) -> A
where
  A: 'static,
  E: Debug + 'static,
  R: 'static,
{
  match run_effect_test_with_env(effect, env).await {
    Ok(value) => value,
    Err(error) => panic!("effectful test failed: {error:?}"),
  }
}

/// Run a `ServiceContext` effect-returning test with a layer and panic on failure.
#[inline]
pub async fn expect_effect_test_with_layer<A, E, ROut>(
  effect: Effect<A, E, ServiceContext>,
  layer: Layer<ROut, E, ()>,
) -> A
where
  A: 'static,
  E: Debug + 'static,
  ROut: 'static,
{
  expect_effect_test(effect.provide(layer)).await
}

/// Run an effect in deterministic test mode and return an `Exit` value.
#[inline]
pub fn run_test<A, E, R>(effect: Effect<A, E, R>, env: R) -> Exit<A, E>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  reset_counters();
  let result = crate::runtime::run_blocking(effect, env);
  assert_hygiene_counters();
  match result {
    Ok(value) => Exit::succeed(value),
    Err(error) => Exit::fail(error),
  }
}

/// Run an effect in deterministic test mode with an explicit test clock.
#[inline]
pub fn run_test_with_clock<A, E, R>(effect: Effect<A, E, R>, env: R, clock: TestClock) -> Exit<A, E>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  let _scope = install_test_clock(clock);
  run_test(effect, env)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scheduling::duration::duration;
  use crate::{MissingService, Schedule, fail, retry, succeed};
  use rstest::rstest;
  use std::sync::Arc;
  use std::sync::atomic::{AtomicUsize, Ordering};

  #[derive(Clone, Debug, PartialEq, effectful::Service)]
  struct TestService {
    value: u32,
  }

  struct TestFailure {
    code: u32,
  }

  impl std::fmt::Debug for TestFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      f.debug_struct("TestFailure")
        .field("code", &self.code)
        .finish()
    }
  }

  fn service_context() -> ServiceContext {
    TestService { value: 9 }.to_context()
  }

  fn service_layer() -> Layer<TestService, MissingService, ()> {
    Layer::succeed(TestService { value: 11 })
  }

  mod run_test {
    use super::*;

    #[test]
    fn run_test_with_success_effect_returns_success_exit() {
      let exit = run_test(succeed::<u32, (), ()>(7), ());
      assert_eq!(exit, Exit::succeed(7));
    }

    #[test]
    fn run_test_with_failure_effect_returns_failure_exit() {
      let exit = run_test(fail::<(), &'static str, ()>("boom"), ());
      assert_eq!(exit, Exit::fail("boom"));
    }

    #[rstest]
    #[case::zero(0u8)]
    #[case::positive(9u8)]
    fn run_test_with_clock_matches_run_test_semantics_for_successful_effect(#[case] value: u8) {
      let effect = succeed::<u8, (), ()>(value);
      let clock = TestClock::new(std::time::Instant::now());
      let exit = run_test_with_clock(effect, (), clock);
      assert_eq!(exit, Exit::succeed(value));
    }

    #[test]
    fn run_test_with_clock_drives_retry_schedule_sleep_without_wall_clock_wait() {
      let start = std::time::Instant::now();
      let clock = TestClock::new(start);
      let attempts = Arc::new(AtomicUsize::new(0));
      let attempts_c = Arc::clone(&attempts);
      let effect = retry(
        move || {
          let attempt = attempts_c.fetch_add(1, Ordering::SeqCst);
          if attempt == 0 {
            fail::<usize, &'static str, ()>("boom")
          } else {
            succeed::<usize, &'static str, ()>(attempt + 1)
          }
        },
        Schedule::spaced(duration::millis(50)).compose(Schedule::recurs(1)),
      );

      let before = std::time::Instant::now();
      let exit = run_test_with_clock(effect, (), clock.clone());
      let elapsed = before.elapsed();

      assert_eq!(exit, Exit::succeed(2));
      assert!(
        elapsed < duration::millis(25),
        "retry waited on wall clock for {elapsed:?}"
      );
      assert_eq!(clock.pending_sleeps(), vec![start + duration::millis(50)]);
    }
  }

  mod effect_test_attribute {
    use super::*;

    #[crate::effect_test]
    fn effect_returning_test_with_unit_environment_passes() -> Effect<(), &'static str, ()> {
      Effect::new(|_| Ok(()))
    }

    #[crate::effect_test(env = "service_context")]
    fn effect_returning_test_with_provided_context_passes()
    -> Effect<(), MissingService, ServiceContext> {
      Effect::<TestService, MissingService, ServiceContext>::service::<TestService>()
        .map(|service| assert_eq!(service.value, 9))
    }

    #[crate::effect_test(layer = "service_layer")]
    fn effect_returning_test_with_provided_layer_passes()
    -> Effect<(), MissingService, ServiceContext> {
      Effect::<TestService, MissingService, ServiceContext>::service::<TestService>()
        .map(|service| assert_eq!(service.value, 11))
    }
  }

  mod async_harness {
    use super::*;

    #[tokio::test]
    async fn run_effect_test_with_env_returns_success_value() {
      let effect = Effect::<u32, MissingService, ServiceContext>::service::<TestService>()
        .map(|service| service.value);

      let result = run_effect_test_with_env(effect, service_context()).await;

      assert_eq!(result, Ok(9));
    }

    #[tokio::test]
    async fn test_runtime_with_env_returns_success_value() {
      let effect = Effect::<u32, MissingService, ServiceContext>::service::<TestService>()
        .map(|service| service.value);

      let result = TestRuntime::with_env(service_context).run(effect).await;

      assert_eq!(result, Ok(9));
    }

    #[tokio::test]
    #[should_panic(expected = "effectful test failed: TestFailure { code: 7 }")]
    async fn expect_effect_test_with_failure_formats_debug_error() {
      expect_effect_test(fail::<(), TestFailure, ()>(TestFailure { code: 7 })).await;
    }
  }

  mod assertions {
    use super::*;

    #[test]
    #[should_panic(expected = "deterministic test harness detected leaked fibers")]
    fn assert_no_leaked_fibers_when_leaked_fiber_recorded_panics() {
      record_leaked_fiber();
      let _ = crate::runtime::run_blocking(assert_no_leaked_fibers(), ());
    }

    #[test]
    #[should_panic(expected = "deterministic test harness detected unclosed scopes")]
    fn assert_no_unclosed_scopes_when_unclosed_scope_recorded_panics() {
      record_unclosed_scope();
      let _ = crate::runtime::run_blocking(assert_no_unclosed_scopes(), ());
    }
  }
}
