//! Immutable persistent hash sets (`im::HashSet`) plus mutable `std::collections::HashSet` newtypes.

use std::collections::HashSet;
use std::hash::Hash;

/// Persistent hash set — backed by [`im::HashSet`].
pub type EffectHashSet<A> = im::HashSet<A>;

/// Empty persistent set.
#[inline]
pub fn empty<A>() -> EffectHashSet<A>
where
  A: Hash + Eq + Clone,
{
  EffectHashSet::new()
}

/// Builds a set from an iterator of elements.
#[inline]
pub fn from_iter<A, I>(iter: I) -> EffectHashSet<A>
where
  I: IntoIterator<Item = A>,
  A: Hash + Eq + Clone,
{
  iter.into_iter().collect()
}

/// Membership test for `value`.
#[inline]
pub fn has<A, Q>(set: &EffectHashSet<A>, value: &Q) -> bool
where
  Q: Hash + Eq + ?Sized,
  A: Hash + Eq + Clone + std::borrow::Borrow<Q>,
{
  set.contains(value)
}

/// Returns a new set including `value`.
#[inline]
pub fn insert<A>(set: &EffectHashSet<A>, value: A) -> EffectHashSet<A>
where
  A: Hash + Eq + Clone,
{
  set.update(value)
}

/// Returns a new set without `value`.
#[inline]
pub fn remove<A, Q>(set: &EffectHashSet<A>, value: &Q) -> EffectHashSet<A>
where
  Q: Hash + Eq + ?Sized,
  A: Hash + Eq + Clone + std::borrow::Borrow<Q>,
{
  set.without(value)
}

/// Insert if absent, remove if present — returns the new set and whether the value is now in the set.
#[inline]
pub fn toggle<A>(set: &EffectHashSet<A>, value: A) -> (EffectHashSet<A>, bool)
where
  A: Hash + Eq + Clone,
{
  if set.contains(&value) {
    (set.without(&value), false)
  } else {
    (set.update(value), true)
  }
}

/// Set union of `left` and `right`.
#[inline]
pub fn union<A>(left: EffectHashSet<A>, right: EffectHashSet<A>) -> EffectHashSet<A>
where
  A: Hash + Eq + Clone,
{
  left.union(right)
}

/// Number of elements.
#[inline]
pub fn size<A>(set: &EffectHashSet<A>) -> usize
where
  A: Hash + Eq + Clone,
{
  set.len()
}

/// True when the set has no elements.
#[inline]
pub fn is_empty<A>(set: &EffectHashSet<A>) -> bool
where
  A: Hash + Eq + Clone,
{
  set.is_empty()
}

/// All elements as a cloned vector (order unspecified).
#[inline]
pub fn values<A>(set: &EffectHashSet<A>) -> Vec<A>
where
  A: Hash + Eq + Clone,
{
  set.iter().cloned().collect()
}

// ── MutableHashSet ───────────────────────────────────────────────────────────

/// In-place mutable set mirroring the immutable helpers.
#[derive(Debug, Clone, Default)]
pub struct MutableHashSet<A>(
  /// Backing standard library set.
  pub HashSet<A>,
);

impl<A: Hash + Eq + Clone> MutableHashSet<A> {
  /// Empty set.
  #[inline]
  pub fn new() -> Self {
    Self(HashSet::new())
  }

  /// Whether `value` is in the set.
  #[inline]
  pub fn has<Q: Hash + Eq + ?Sized>(&self, value: &Q) -> bool
  where
    A: std::borrow::Borrow<Q>,
  {
    self.0.contains(value)
  }

  /// Adds `value` to the set.
  #[inline]
  pub fn insert(&mut self, value: A) {
    self.0.insert(value);
  }

  /// Removes `value`; returns whether it was present.
  #[inline]
  pub fn remove<Q: Hash + Eq + ?Sized>(&mut self, value: &Q) -> bool
  where
    A: std::borrow::Borrow<Q>,
  {
    self.0.remove(value)
  }

  /// Insert-if-absent / remove-if-present; returns whether `value` is now in the set.
  #[inline]
  pub fn toggle(&mut self, value: A) -> bool {
    if self.0.remove(&value) {
      false
    } else {
      self.0.insert(value);
      true
    }
  }

  /// Element count.
  #[inline]
  pub fn size(&self) -> usize {
    self.0.len()
  }

  /// True when empty.
  #[inline]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn hash_set_toggle_adds_then_removes() {
    let s = empty::<i32>();
    let (s, now_in) = toggle(&s, 5);
    assert!(now_in);
    assert!(has(&s, &5));
    let (s, now_in) = toggle(&s, 5);
    assert!(!now_in);
    assert!(!has(&s, &5));
  }

  #[test]
  fn mutable_set_toggle_matches_immutable_semantics() {
    let mut m = MutableHashSet::new();
    assert!(m.toggle(7));
    assert!(m.has(&7));
    assert!(!m.toggle(7));
    assert!(!m.has(&7));
  }
}
