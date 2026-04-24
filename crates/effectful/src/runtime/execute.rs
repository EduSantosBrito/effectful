//! Synchronous and async execution functions: `run_blocking`, `run_async`, `Never` (§ 6.2).
//!
//! These are the lowest-level interpreters for [`crate::kernel::Effect`], composing directly
//! from Stratum 2 primitives without any fiber or runtime-trait dependency.

use core::convert::Infallible;

use crate::kernel::{
  BoxFuture, Effect,
  effect::{SyncStep, start_async_operation, start_effect},
};

/// Effect.ts-style uninhabited error marker for infallible runtime operations.
pub type Never = Infallible;

/// Run an effect to completion on the current thread using a tight poll loop.
///
/// The future from [`Effect::run`] is polled with `Waker::noop`. If a step returns
/// `Poll::Pending`, this implementation calls `std::thread::yield_now` and polls again — there is
/// no real async driver, so effects that genuinely need to await I/O or other wakeups may spin or
/// stall. For those cases use [`run_async`] (or an executor) instead.
#[inline(always)]
pub fn run_blocking<A, E, R>(effect: Effect<A, E, R>, mut env: R) -> Result<A, E>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  let waker = std::task::Waker::noop();
  let mut cx = std::task::Context::from_waker(waker);
  let mut fut: BoxFuture<'_, Result<A, E>> = match start_effect(effect, &mut env) {
    SyncStep::Ready(output) => return output,
    SyncStep::AsyncBorrow(f) => start_async_operation(f, &mut env),
    SyncStep::AsyncStatic(fut) => fut,
    SyncStep::AsyncPoll(mut poller) => loop {
      match poller(&mut env, &mut cx) {
        std::task::Poll::Ready(output) => return output,
        std::task::Poll::Pending => std::thread::yield_now(),
      }
    },
  };
  loop {
    match fut.as_mut().poll(&mut cx) {
      std::task::Poll::Ready(output) => return output,
      std::task::Poll::Pending => std::thread::yield_now(),
    }
  }
}

/// Run an effect to completion using the async executor (`.await` on [`Effect::run`]).
#[inline(always)]
pub async fn run_async<A, E, R>(effect: Effect<A, E, R>, mut env: R) -> Result<A, E>
where
  A: 'static,
  E: 'static,
  R: 'static,
{
  match start_effect(effect, &mut env) {
    SyncStep::Ready(output) => output,
    SyncStep::AsyncBorrow(f) => start_async_operation(f, &mut env).await,
    SyncStep::AsyncStatic(fut) => fut.await,
    SyncStep::AsyncPoll(mut poller) => core::future::poll_fn(move |cx| poller(&mut env, cx)).await,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::kernel::succeed;

  mod run_blocking {
    use super::*;

    #[test]
    fn with_success_effect_returns_ok_value() {
      let result = run_blocking(succeed::<u8, (), ()>(9), ());
      assert_eq!(result, Ok(9));
    }

    #[test]
    fn completes_flat_mapped_effect_without_double_boxing() {
      let eff = crate::kernel::succeed::<u8, &'static str, ()>(3)
        .flat_map(|n| crate::kernel::succeed(n + 1));
      assert_eq!(run_blocking(eff, ()), Ok(4));
    }

    #[test]
    fn completes_sync_only_chain_with_multiple_combinators() {
      let eff = crate::kernel::succeed::<u8, &'static str, ()>(3)
        .flat_map(|n| crate::kernel::succeed(n + 1))
        .and_then_discard(crate::kernel::succeed::<u8, &'static str, ()>(9))
        .map(|n| n * 2);
      assert_eq!(run_blocking(eff, ()), Ok(8));
    }

    #[test]
    fn with_failure_effect_returns_err_value() {
      let result = run_blocking(crate::kernel::fail::<u8, &str, ()>("boom"), ());
      assert_eq!(result, Err("boom"));
    }
  }

  mod run_async {
    use super::*;

    #[test]
    fn with_success_effect_returns_ok_value() {
      let async_out = pollster::block_on(run_async(succeed::<u8, (), ()>(11), ()));
      assert_eq!(async_out, Ok(11));
    }

    #[test]
    fn completes_sync_prefix_then_async_boundary_then_sync_tail() {
      let eff = crate::kernel::succeed::<u8, &'static str, ()>(1)
        .flat_map(|n| {
          crate::kernel::from_async(move |_r| async move { Ok::<u8, &'static str>(n + 1) })
        })
        .and_then_discard(crate::kernel::succeed::<u8, &'static str, ()>(99))
        .map(|n| n + 1);
      let async_out = pollster::block_on(run_async(eff, ()));
      assert_eq!(async_out, Ok(3));
    }
  }
}

#[cfg(test)]
mod yield_tests {
  use super::*;
  use crate::kernel::{Effect, from_async};
  use std::time::Instant;

  #[tokio::test(flavor = "current_thread")]
  async fn yield_effect_actually_suspends() {
    let eff = from_async(|_r| async move {
      tokio::task::yield_now().await;
      Ok::<(), ()>(())
    })
    .map(|_| 42);

    let start = Instant::now();
    let result = run_async(eff, ()).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result, 42);
    assert!(
      elapsed.as_micros() > 1,
      "Should have taken more than 1us due to yield, took {:?}",
      elapsed
    );
  }

  #[tokio::test(flavor = "current_thread")]
  async fn flat_map_over_yield_actually_suspends() {
    let eff = Effect::new(|_| Ok::<u64, ()>(1u64))
      .flat_map(|n| {
        from_async(|_r| async move {
          tokio::task::yield_now().await;
          Ok::<(), ()>(())
        })
        .map(move |_| n + 1)
      })
      .flat_map(|n| {
        from_async(|_r| async move {
          tokio::task::yield_now().await;
          Ok::<(), ()>(())
        })
        .map(move |_| n + 1)
      });

    let start = Instant::now();
    let result = run_async(eff, ()).await.unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result, 3);
    assert!(
      elapsed.as_micros() > 1,
      "Should have taken more than 1us due to yields, took {:?}",
      elapsed
    );
  }
}
