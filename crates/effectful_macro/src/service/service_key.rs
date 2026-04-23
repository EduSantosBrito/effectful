//! `service_key!` macro.

/// Declare a zero-sized service key type (Effect.ts tag identity).
///
/// Generated structs use **`PartialEq` / `Eq` / `Hash`** derives so they participate in
/// [`Equal`](::effectful::Equal) and [`EffectHash`](::effectful::EffectHash) via the blanket impls—the
/// same structural equality story as [`Brand`](::effectful::Brand) for nominal, type-level tags (all
/// values of a given key type are equal; distinct key types are distinct at compile time).
///
/// ```ignore
/// effectful::service_key!(/// Pool handle
/// pub struct PgPoolKey);
/// type PgPoolSvc = effectful::Service<PgPoolKey, sqlx::PgPool>;
/// ```
#[macro_export]
#[deprecated(
  since = "0.2.0",
  note = "Use `#[derive(Service)]` instead. `service_key!` will be removed in a future release."
)]
macro_rules! service_key {
  ($(#[$m:meta])* $vis:vis struct $name:ident) => {
    $(#[$m])*
    #[doc = "Nominal service key (`service_key!`). Implements [`Equal`](::effectful::Equal) and [`EffectHash`](::effectful::EffectHash) through the standard derives—Brand-style structural equality for this ZST tag."]
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    $vis struct $name;
  };
}
