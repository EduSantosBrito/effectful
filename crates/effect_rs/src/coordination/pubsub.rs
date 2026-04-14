//! Broadcast pub/sub hub backed by [`tokio::sync::broadcast`].
//!
//! [`PubSub::subscribe`] returns a [`Queue`] filled by a background forwarder. A [`Scope`]
//! finalizer aborts the task and shuts down the queue when the scope used as the effect
//! environment closes.

use std::sync::Arc;

use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::broadcast;
use tokio::sync::watch;

use crate::coordination::queue::Queue;
use crate::kernel::{Effect, box_future, succeed};
use crate::resource::scope::Scope;
use crate::runtime::{Never, run_blocking};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PubSubMode {
  /// [`PubSub::publish`] returns `false` when the buffer is full (`len >= capacity`).
  Bounded,
  /// Same backpressure rule as [`PubSubMode::Bounded`] (incoming value not stored when full).
  Dropping,
  /// Oldest values are evicted by the broadcast channel when the ring is full.
  Sliding,
  /// Large internal ring; [`PubSub::publish`] does not fail due to buffer size.
  Unbounded,
}

struct PubSubInner<A: Send + Clone + 'static> {
  tx: AsyncMutex<Option<broadcast::Sender<A>>>,
  capacity: usize,
  mode: PubSubMode,
  shutdown_tx: watch::Sender<bool>,
}

/// Cloneable broadcast hub: each [`subscribe`](PubSub::subscribe) gets a [`Queue`] of all published
/// values (within lag / capacity policy).
#[derive(Clone)]
pub struct PubSub<A: Send + Clone + 'static> {
  inner: Arc<PubSubInner<A>>,
}

/// Internal capacity for [`PubSub::unbounded`].
const UNBOUNDED_CAP: usize = 65_536;

impl<A: Send + Clone + 'static> PubSub<A> {
  fn new(tx: broadcast::Sender<A>, capacity: usize, mode: PubSubMode) -> Self {
    let shutdown_tx = watch::channel(false).0;
    Self {
      inner: Arc::new(PubSubInner {
        tx: AsyncMutex::new(Some(tx)),
        capacity,
        mode,
        shutdown_tx,
      }),
    }
  }

  /// Bounded hub: [`PubSub::publish`] returns `false` when the ring already holds `capacity` messages.
  pub fn bounded(capacity: usize) -> Effect<Self, (), ()> {
    let cap = capacity.max(1);
    let (tx, _) = broadcast::channel(cap);
    succeed(Self::new(tx, cap, PubSubMode::Bounded))
  }

  /// Hub with a large fixed ring; publish does not fail for buffer fullness.
  pub fn unbounded() -> Effect<Self, (), ()> {
    let (tx, _) = broadcast::channel(UNBOUNDED_CAP);
    succeed(Self::new(tx, UNBOUNDED_CAP, PubSubMode::Unbounded))
  }

  /// Same ring size as [`PubSub::bounded`]; publish rejects when the buffer is full (newest not stored).
  pub fn dropping(capacity: usize) -> Effect<Self, (), ()> {
    let cap = capacity.max(1);
    let (tx, _) = broadcast::channel(cap);
    succeed(Self::new(tx, cap, PubSubMode::Dropping))
  }

  /// Ring of size `capacity`; when full, oldest messages are dropped on each new publish.
  pub fn sliding(capacity: usize) -> Effect<Self, (), ()> {
    let cap = capacity.max(1);
    let (tx, _) = broadcast::channel(cap);
    succeed(Self::new(tx, cap, PubSubMode::Sliding))
  }

  /// Logical capacity configured for this hub (broadcast ring length).
  #[inline]
  pub fn capacity(&self) -> usize {
    self.inner.capacity
  }

  /// `true` once [`PubSub::shutdown`] has run.
  #[inline]
  pub fn is_shutdown(&self) -> bool {
    *self.inner.shutdown_tx.borrow()
  }

  /// Close the hub; further [`publish`](PubSub::publish) calls fail and new [`subscribe`](PubSub::subscribe) calls get a shut-down queue.
  pub fn shutdown(&self) -> Effect<(), (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let mut guard = inner.tx.lock().await;
        guard.take();
        drop(guard);
        let _ = inner.shutdown_tx.send(true);
        Ok(())
      })
    })
  }

  /// Wait until [`PubSub::shutdown`] has been observed.
  pub fn await_shutdown(&self) -> Effect<(), (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        if *inner.shutdown_tx.borrow() {
          return Ok(());
        }
        let mut rx = inner.shutdown_tx.subscribe();
        let _ = rx.changed().await;
        Ok(())
      })
    })
  }

  /// Messages currently retained in the broadcast ring (requires the hub not to be shut down).
  pub fn size(&self) -> Effect<usize, (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let guard = inner.tx.lock().await;
        let Some(tx) = guard.as_ref() else {
          return Ok(0);
        };
        Ok(tx.len())
      })
    })
  }

  /// `true` when the broadcast ring has no retained messages (or the hub is shut down).
  pub fn is_empty(&self) -> Effect<bool, (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let guard = inner.tx.lock().await;
        let Some(tx) = guard.as_ref() else {
          return Ok(true);
        };
        Ok(tx.is_empty())
      })
    })
  }

  /// `true` when the ring holds [`PubSub::capacity`] messages (same idea as [`Queue::is_full`]).
  pub fn is_full(&self) -> Effect<bool, (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let guard = inner.tx.lock().await;
        let Some(tx) = guard.as_ref() else {
          return Ok(true);
        };
        Ok(tx.len() >= inner.capacity)
      })
    })
  }

  /// Enqueue one message for all active subscribers. `false` if shut down, no receivers, or buffer full (bounded/dropping).
  pub fn publish(&self, value: A) -> Effect<bool, (), ()> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let guard = inner.tx.lock().await;
        let Some(tx) = guard.as_ref() else {
          return Ok(false);
        };
        match inner.mode {
          PubSubMode::Bounded | PubSubMode::Dropping => {
            if tx.len() >= inner.capacity {
              return Ok(false);
            }
          }
          PubSubMode::Sliding | PubSubMode::Unbounded => {}
        }
        match tx.send(value) {
          Ok(_) => Ok(true),
          Err(_) => Ok(false),
        }
      })
    })
  }

  /// Publish in order; returns values that were not sent (shut down, no receivers, or bounded/dropping full).
  pub fn publish_all<I>(&self, iter: I) -> Effect<Vec<A>, (), ()>
  where
    I: IntoIterator<Item = A> + 'static,
    I::IntoIter: Send + 'static,
  {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |_r| {
      box_future(async move {
        let mut left = Vec::new();
        for v in iter {
          let guard = inner.tx.lock().await;
          let Some(tx) = guard.as_ref() else {
            left.push(v);
            continue;
          };
          let can_send = match inner.mode {
            PubSubMode::Bounded | PubSubMode::Dropping => tx.len() < inner.capacity,
            PubSubMode::Sliding | PubSubMode::Unbounded => true,
          };
          if !can_send {
            left.push(v);
            continue;
          }
          match tx.send(v) {
            Ok(_) => {}
            Err(e) => left.push(e.0),
          }
        }
        Ok(left)
      })
    })
  }

  /// Subscribe as a [`Queue`]. Run with [`Scope`] as the environment; when that scope closes, the
  /// broadcast receiver is dropped and the forward task is aborted.
  pub fn subscribe(&self) -> Effect<Queue<A>, Never, Scope> {
    let inner = Arc::clone(&self.inner);
    Effect::new_async(move |scope: &mut Scope| {
      let scope_for_fin = scope.clone();
      box_future(async move {
        let q = run_blocking(Queue::unbounded(), ()).expect("queue");
        let brx = {
          let guard = inner.tx.lock().await;
          guard.as_ref().map(|tx| tx.subscribe())
        };
        let Some(mut brx) = brx else {
          let _ = run_blocking(q.shutdown(), ());
          return Ok(q);
        };

        let q_task = q.clone();
        let handle = tokio::spawn(async move {
          loop {
            match brx.recv().await {
              Ok(v) => {
                let _ = run_blocking(q_task.offer(v), ());
              }
              Err(broadcast::error::RecvError::Lagged(_)) => continue,
              Err(broadcast::error::RecvError::Closed) => break,
            }
          }
          let _ = run_blocking(q_task.shutdown(), ());
        });

        let q_fin = q.clone();
        let _ = scope_for_fin.add_finalizer(Box::new(move |_exit| {
          Effect::new_async(move |_r: &mut ()| {
            let h = handle;
            let q = q_fin.clone();
            box_future(async move {
              h.abort();
              let _ = run_blocking(q.shutdown(), ());
              Ok(())
            })
          })
        }));

        Ok(q)
      })
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::run_async;
  use std::time::Duration;

  #[tokio::test]
  async fn pubsub_subscriber_receives_all_messages() {
    let ps = run_async(PubSub::<u32>::bounded(8), ())
      .await
      .expect("pubsub");
    let scope = Scope::make();
    let q = run_async(ps.subscribe(), scope.clone())
      .await
      .expect("subscribe");
    assert!(run_async(ps.publish(1), ()).await.expect("pub"));
    assert!(run_async(ps.publish(2), ()).await.expect("pub"));
    assert!(run_async(ps.publish(3), ()).await.expect("pub"));
    for want in [1u32, 2, 3] {
      tokio::task::yield_now().await;
      assert_eq!(run_async(q.take(), ()).await.expect("take"), want);
    }
    scope.close();
  }

  #[tokio::test]
  async fn pubsub_subscribe_auto_unsubscribes_on_scope_close() {
    let ps = run_async(PubSub::<u32>::bounded(8), ())
      .await
      .expect("pubsub");
    let scope = Scope::make();
    let _q = run_async(ps.clone().subscribe(), scope.clone())
      .await
      .expect("subscribe");
    assert!(scope.close());
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert!(
      !run_async(ps.publish(99), ()).await.expect("pub"),
      "no active receivers after scope close"
    );
  }

  #[tokio::test]
  async fn pubsub_sliding_drops_oldest_for_slow_subscriber() {
    let ps = run_async(PubSub::<u32>::sliding(2), ())
      .await
      .expect("pubsub");
    let scope = Scope::make();
    let q = run_async(ps.subscribe(), scope.clone())
      .await
      .expect("subscribe");
    for i in 1..=3u32 {
      assert!(run_async(ps.publish(i), ()).await.expect("pub"));
    }
    tokio::time::sleep(Duration::from_millis(50)).await;
    let first = run_async(q.take(), ()).await.expect("take");
    assert_ne!(
      first, 1,
      "with sliding capacity 2, value 1 should be evicted before slow subscriber catches up"
    );
    scope.close();
  }
}
