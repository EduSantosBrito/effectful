# The bind* Operator Explained

The `bind*` (bind-star) is the bind operator inside `effect!`. It means: "execute this effect and give me its success value; if it fails, propagate the failure and stop."

## Basic Usage

```rust,ignore
effect! {
    let user = bind* fetch_user(42);   // bind the result to `user`
    user.name
}
```

`bind* fetch_user(42)` desugars to a `flat_map`. The rest of the block becomes the body of the closure.

## Discarding Results

When you don't need the value, use `bind*` without a binding:

```rust,ignore
effect! {
    bind* log_event("processing started");   // run for side effect, discard result
    let result = bind* do_work();
    bind* log_event("processing done");
    result
}
```

Both `bind* log_event(...)` expressions run for their effects and the `()` return is discarded.

## Method Calls on Effects

`bind*` works on any expression that evaluates to an `Effect`. That includes method chains:

```rust,ignore
effect! {
    let user = bind* fetch_user(id).map_error(AppError::Database);
    let posts = bind* retry(
        || fetch_posts(user.id).map_error(AppError::Database),
        Schedule::exponential(Duration::from_millis(100)).compose(Schedule::recurs(3)),
    );
    (user, posts)
}
```

The `bind*` applies to the entire expression that follows it. For retry/repeat, use the free functions that return an `Effect`.

## bind* in Conditionals and Loops

You can use `bind*` inside `if` expressions and loops:

```rust,ignore
effect! {
    let value = if condition {
        bind* compute_a()
    } else {
        bind* compute_b()
    };
    process(value)
}
```

Both branches are effects; the macro handles either path.

```rust,ignore
effect! {
    for id in user_ids {
        bind* process_user(id);  // sequential: one at a time
    }
    "done"
}
```

Note: this is *sequential* iteration. For concurrent processing, use `fiber_all` (Chapter 9).

## What bind* Cannot Do

`bind*` only works *inside* an `effect!` block. Calling it outside is a compile error:

```rust,ignore
// Does not compile — bind* is not valid here
let x = bind* fetch_user(42);

// Must be inside effect!
let x = effect! { bind* fetch_user(42) };
```

Also, `bind*` cannot bind across an async closure boundary. If you're calling `from_async`, the body of the async block is separate:

```rust,ignore
effect! {
    let result = bind* from_async(|_r| async move {
        // Inside here, you're in regular Rust async — no bind*.
        let data = some_future().await?;
        Ok(data)
    });
    result
}
```

Use `bind*` outside the `async move` block; use `.await` inside it.

## The Old Postfix Syntax (Deprecated)

Early versions of effectful used a postfix bind-star: `expr ~`. This is no longer valid. Always use the prefix form:

```rust,ignore
// OLD — do not use
step_a() ~;

// GOOD
bind* step_a();
let x = bind* step_b();
```

If you see postfix bind-star in older code, update it to the prefix form.
