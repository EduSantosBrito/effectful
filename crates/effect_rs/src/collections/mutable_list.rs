//! Mutable deque-backed list — mirrors Effect.ts `MutableList` style API.

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::streaming::chunk::Chunk;

/// A mutex-backed double-ended list.
pub struct MutableList<A> {
  inner: Mutex<VecDeque<A>>,
}

impl<A> MutableList<A> {
  /// Empty list.
  #[inline]
  pub fn make() -> Self {
    Self {
      inner: Mutex::new(VecDeque::new()),
    }
  }

  /// Push `value` at the back.
  #[inline]
  pub fn append(&self, value: A) {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .push_back(value);
  }

  /// Push `value` at the front.
  #[inline]
  pub fn prepend(&self, value: A) {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .push_front(value);
  }

  /// First element, if any.
  #[inline]
  pub fn head(&self) -> Option<A>
  where
    A: Clone,
  {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .front()
      .cloned()
  }

  /// All elements after the first (empty when length ≤ 1).
  #[inline]
  pub fn tail(&self) -> Chunk<A>
  where
    A: Clone,
  {
    let g = self.inner.lock().expect("mutable_list mutex poisoned");
    if g.len() <= 1 {
      Chunk::empty()
    } else {
      Chunk::from_vec(g.iter().skip(1).cloned().collect())
    }
  }

  /// Last element, if any.
  #[inline]
  pub fn last(&self) -> Option<A>
  where
    A: Clone,
  {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .back()
      .cloned()
  }

  /// Remove and return the last element.
  #[inline]
  pub fn pop(&self) -> Option<A> {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .pop_back()
  }

  /// Remove and return the first element.
  #[inline]
  pub fn shift(&self) -> Option<A> {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .pop_front()
  }

  /// Snapshot of the whole list as an immutable [`Chunk`].
  #[inline]
  pub fn to_chunk(&self) -> Chunk<A>
  where
    A: Clone,
  {
    let g = self.inner.lock().expect("mutable_list mutex poisoned");
    Chunk::from_vec(g.iter().cloned().collect())
  }

  /// Number of elements.
  #[inline]
  pub fn length(&self) -> usize {
    self
      .inner
      .lock()
      .expect("mutable_list mutex poisoned")
      .len()
  }

  /// Invokes `f` for each element in order (holds the mutex for the whole walk).
  #[inline]
  pub fn for_each(&self, mut f: impl FnMut(&A)) {
    let g = self.inner.lock().expect("mutable_list mutex poisoned");
    for x in g.iter() {
      f(x);
    }
  }
}

/// Accumulate elements with [`MutableList`] then freeze into [`Chunk`].
pub struct ChunkBuilder<A> {
  list: MutableList<A>,
}

impl<A> ChunkBuilder<A> {
  /// Empty builder.
  #[inline]
  pub fn new() -> Self {
    Self {
      list: MutableList::make(),
    }
  }

  /// Append `value` in list order.
  #[inline]
  pub fn append(&self, value: A) {
    self.list.append(value);
  }

  /// Freeze accumulated values into a [`Chunk`].
  #[inline]
  pub fn to_chunk(&self) -> Chunk<A>
  where
    A: Clone,
  {
    self.list.to_chunk()
  }
}

impl<A> Default for ChunkBuilder<A> {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mutable_list_shift_removes_head() {
    let list = MutableList::<i32>::make();
    list.append(1);
    list.append(2);
    assert_eq!(list.shift(), Some(1));
    assert_eq!(list.head(), Some(2));
    assert_eq!(list.length(), 1);
  }

  #[test]
  fn mutable_list_tail_skips_first_element() {
    let list = MutableList::<i32>::make();
    list.append(10);
    list.append(20);
    list.append(30);
    let t = list.tail();
    assert_eq!(t.len(), 2);
  }

  #[test]
  fn chunk_builder_via_mutable_list_preserves_order() {
    let b = ChunkBuilder::<i32>::new();
    b.append(1);
    b.append(2);
    b.append(3);
    assert_eq!(b.to_chunk().into_vec(), vec![1, 2, 3]);
  }
}
