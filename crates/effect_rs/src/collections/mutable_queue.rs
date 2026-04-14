//! Bounded and unbounded FIFO queues.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::streaming::chunk::Chunk;

/// FIFO queue; bounded variant drops new items when full.
pub struct MutableQueue<A> {
  inner: Mutex<VecDeque<A>>,
  capacity: Option<usize>,
}

impl<A> MutableQueue<A> {
  /// Unbounded queue.
  #[inline]
  pub fn unbounded() -> Self {
    Self {
      inner: Mutex::new(VecDeque::new()),
      capacity: None,
    }
  }

  /// Bounded queue with at most `capacity` elements (0 allows nothing).
  #[inline]
  pub fn bounded(capacity: usize) -> Self {
    Self {
      inner: Mutex::new(VecDeque::new()),
      capacity: Some(capacity),
    }
  }

  /// `Some(n)` for bounded queues, `None` when unbounded.
  #[inline]
  pub fn capacity(&self) -> Option<usize> {
    self.capacity
  }

  /// Current number of queued elements.
  #[inline]
  pub fn length(&self) -> usize {
    self
      .inner
      .lock()
      .expect("mutable_queue mutex poisoned")
      .len()
  }

  /// `true` when [`MutableQueue::length`] is zero.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.length() == 0
  }

  /// `true` when bounded and at capacity (always `false` for unbounded queues).
  #[inline]
  pub fn is_full(&self) -> bool {
    match self.capacity {
      Some(c) => self.length() >= c,
      None => false,
    }
  }

  /// Enqueue `value`. Returns `false` if bounded and full (value is dropped).
  #[inline]
  pub fn offer(&self, value: A) -> bool {
    let mut g = self.inner.lock().expect("mutable_queue mutex poisoned");
    if let Some(c) = self.capacity
      && g.len() >= c
    {
      return false;
    }
    g.push_back(value);
    true
  }

  /// Enqueue all values in order; stops at first reject when bounded and full.
  #[inline]
  pub fn offer_all(&self, iter: impl IntoIterator<Item = A>) -> usize {
    let mut n = 0usize;
    for v in iter {
      if !self.offer(v) {
        break;
      }
      n += 1;
    }
    n
  }

  /// Dequeue the front element, or `default()` if empty.
  #[inline]
  pub fn poll(&self, default: impl FnOnce() -> A) -> A {
    self
      .inner
      .lock()
      .expect("mutable_queue mutex poisoned")
      .pop_front()
      .unwrap_or_else(default)
  }

  /// Dequeue up to `max` elements into a [`Chunk`] (front-first).
  #[inline]
  pub fn poll_up_to(&self, max: usize) -> Chunk<A>
  where
    A: Clone,
  {
    let mut g = self.inner.lock().expect("mutable_queue mutex poisoned");
    let mut out = Vec::new();
    for _ in 0..max {
      if let Some(x) = g.pop_front() {
        out.push(x);
      } else {
        break;
      }
    }
    Chunk::from_vec(out)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mutable_queue_offer_drops_when_full_bounded() {
    let q = MutableQueue::<i32>::bounded(2);
    assert!(q.offer(1));
    assert!(q.offer(2));
    assert!(!q.offer(3));
    assert_eq!(q.length(), 2);
    assert!(q.is_full());
  }

  #[test]
  fn mutable_queue_unbounded_never_full() {
    let q = MutableQueue::<u8>::unbounded();
    assert!(q.offer(1));
    assert!(!q.is_full());
    assert_eq!(q.capacity(), None);
  }
}
