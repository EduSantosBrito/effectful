# Why Do-Notation Exists

Consider three steps that each depend on the previous result:

```rust,ignore
fn step_a() -> Effect<i32, Err, ()>   { succeed(1) }
fn step_b(n: i32) -> Effect<i32, Err, ()> { succeed(n * 2) }
fn step_c(n: i32) -> Effect<String, Err, ()> { succeed(n.to_string()) }
```

Written with raw `flat_map`:

```rust,ignore
let program = step_a()
    .flat_map(|a| step_b(a)
        .flat_map(|b| step_c(b)));
```

Two steps: readable. Five steps: a pyramid. Ten steps: indistinguishable from callback hell.

Haskell solved this decades ago with *do-notation*. Scala's for-comprehensions do the same thing. Rust doesn't have built-in do-notation, so effectful provides it via a macro.

## Do-Notation as a Concept

Do-notation lets you write sequential effectful code that *looks* like imperative code:

```text
do
  a ← step_a
  b ← step_b(a)
  c ← step_c(b)
  return c
```

Each `←` means "run this effect and bind its result to this name." If any step fails, the whole computation short-circuits.

Rust can't use the `←` symbol, so effectful uses `bind*` (prefix bind-star):

```rust,ignore
effect! {
    let a = bind* step_a();
    let b = bind* step_b(a);
    let c = bind* step_c(b);
    c
}
```

Same semantics. Rust syntax. Zero nesting.

## How the Desugaring Works

The macro transforms each `bind* expr` into a `flat_map`:

```rust,ignore
// Written:
effect! {
    let a = bind* step_a();
    let b = bind* step_b(a);
    b.to_string()
}

// Roughly expands to:
step_a().flat_map(|a| {
    step_b(a).flat_map(|b| {
        succeed(b.to_string())
    })
})
```

The macro generates exactly the nested `flat_map` chain you'd write by hand — just without the visual noise.

## One Body, One block

One discipline matters: **use one `effect!` block per function**. Don't branch between two macro bodies:

```rust,ignore
// BAD — two separate effect! blocks for one computation
if flag {
    effect! { let x = bind* a(); x }
} else {
    effect! { let y = bind* b(); y }
}

// GOOD — one block, branching inside
effect! {
    if flag {
        bind* a()
    } else {
        bind* b()
    }
}
```

A single `effect!` block is a single description. Splitting it into multiple blocks loses the composition guarantee.

## Pure Expressions

Not every line inside `effect!` has to be an effect. Pure Rust expressions work normally:

```rust,ignore
effect! {
    let user = bind* fetch_user(id);
    let name = user.name.to_uppercase();  // pure — no bind*     let posts = bind* fetch_posts(user.id);
    (name, posts)
}
```

Only use `bind*` when the expression has type `Effect<_, _, _>`. Pure expressions just run inline.
