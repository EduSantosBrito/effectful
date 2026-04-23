# Property Testing — Invariants over Inputs

Property tests check invariants over many generated inputs. Effect programs are good targets because the runner boundary is explicit and test environments are ordinary values.

## Setup

```toml
[dev-dependencies]
proptest = "1"
```

## Testing Pure Effects

```rust,ignore
use effectful::{Exit, run_test};
use proptest::prelude::*;

proptest! {
    #[test]
    fn addition_is_commutative(a: i64, b: i64) {
        let r_ab = run_test(add(a, b), ());
        let r_ba = run_test(add(b, a), ());

        prop_assert_eq!(r_ab, r_ba);
    }
}
```

`run_test(effect, env)` returns `Exit<A, E>`. For properties, either compare exits directly or match `Exit::Success(value)`.

## Testing Schema Round-Trips

Schemas expose `encode`, `decode`, and `decode_unknown`.

```rust,ignore
use effectful::schema::{i64, string, struct_, transform};
use proptest::prelude::*;

proptest! {
    #[test]
    fn user_schema_round_trips(name in "[a-zA-Z]{1,50}", age in 0i64..=120) {
        let schema = transform(
            struct_("name", string::<()>(), "age", i64::<()>()),
            |(name, age)| Ok(User { name, age }),
            |user: User| (user.name, user.age),
        );

        let original = User { name, age };
        let wire = schema.encode(original.clone());
        let parsed = schema.decode(wire);

        prop_assert_eq!(parsed.ok(), Some(original));
    }
}
```

Round-trip tests catch asymmetries between encode and decode logic.

## Testing Error Invariants

Use `TRef::make` and `commit` to construct transactional state.

```rust,ignore
use effectful::{Cause, Exit, TRef, commit, run_blocking, run_test};
use proptest::prelude::*;

proptest! {
    #[test]
    fn withdraw_never_goes_negative(balance in 0u64..=1_000_000, amount in 0u64..=1_000_000) {
        let account = run_blocking(commit(TRef::make(balance)), ()).expect("make account");
        let exit = run_test(withdraw(account.clone(), amount), ());
        let new_balance = run_blocking(commit(account.read_stm::<InsufficientFunds>()), ())
            .expect("read balance");

        if amount <= balance {
            prop_assert!(matches!(exit, Exit::Success(_)));
            prop_assert_eq!(new_balance, balance - amount);
        } else {
            prop_assert!(matches!(exit, Exit::Failure(Cause::Fail(InsufficientFunds))));
            prop_assert_eq!(new_balance, balance);
        }
    }
}
```

## Generating Service Environments

For integration-style properties, generate random state and place it in a test service.

```rust,ignore
proptest! {
    #[test]
    fn get_user_returns_what_was_saved(user in arbitrary_user()) {
        let db = Db::in_memory();
        let env = db.clone().to_context();

        let save_exit = run_test(save_user(user.clone()), env.clone());
        prop_assert!(matches!(save_exit, Exit::Success(_)));

        let get_exit = run_test(get_user(user.id), env);
        prop_assert!(matches!(get_exit, Exit::Success(retrieved) if retrieved == user));
    }
}
```

Define generators with normal `proptest` strategies.

```rust,ignore
fn arbitrary_user() -> impl Strategy<Value = User> {
    (
        any::<u64>().prop_map(UserId::new),
        "[a-zA-Z ]{1,50}",
        0i64..=120,
    ).prop_map(|(id, name, age)| User { id, name, age })
}
```

## Schema-Driven Generation

There is no built-in `generate_valid::<T>()` helper. Keep generators beside schemas and use round-trip properties to ensure they stay aligned.

```rust,ignore
proptest! {
    #[test]
    fn generated_users_are_accepted(user in arbitrary_user()) {
        let schema = user_schema();
        let wire = schema.encode(user.clone());
        prop_assert_eq!(schema.decode(wire).ok(), Some(user));
    }
}
```

## Shrinking

`proptest` automatically shrinks failing inputs to the smallest example that still fails. Because effects are run explicitly, shrinking remains easy to reason about.

```text
Test failed. Minimal failing input:
  name = ""
  age = -1
Reason: must not be empty (path: name)
```
