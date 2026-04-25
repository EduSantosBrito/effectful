# API Quick Reference

A condensed reference for commonly used items in `effectful` 0.2.2. For complete signatures, run `cargo doc --open -p effectful`.

## Core Types

| Type | Description |
|------|-------------|
| `Effect<A, E, R>` | Lazy computation that succeeds with `A`, fails with `E`, and requires environment `R` |
| `Exit<A, E>` | Terminal outcome: `Success(A)` or `Failure(Cause<E>)` |
| `Cause<E>` | Failure algebra: `Fail(E)`, `Die(String)`, `Interrupt(FiberId)`, `Both`, `Then` |
| `Context<Cons<..., Nil>>` | Typed heterogeneous environment for tagged services |
| `ServiceContext` | Runtime service table used by the derive-based service/layer API |
| `Layer<ROut, E, RIn>` | Lazy service layer for `ServiceContext` |
| `Stm<A, E>` | Transactional computation |
| `Stream<A, E, R>` | Pull stream of `A` chunks |
| `Sink<Out, In, E, R>` | Consumer of stream elements |
| `Schema<A, I, E>` | Bidirectional decoder/encoder with `Unknown` support |

## Effect Constructors

| Function | Notes |
|----------|-------|
| `succeed(a)` | Always succeeds with `a` |
| `fail(e)` | Always fails with `e` |
| `pure(a)` | Infallible `Effect<A, (), ()>` convenience |
| `from_async(f)` | Lift an async closure returning `Result<A, E>` |
| `Effect::new(f)` | Lift a synchronous closure over `&mut R` |
| `Effect::new_async(f)` | Lift an async closure that may borrow `&mut R` |
| `effect! { ... }` | Do-notation; use `bind* effect` to bind |

## Effect Combinators

| Method | Notes |
|--------|-------|
| `.map(f)` | Transform success value |
| `.flat_map(f)` | Sequentially compose effects |
| `.and_then(other)` | Sequence and keep right result |
| `.and_then_discard(other)` | Sequence and keep left result |
| `.map_error(f)` | Transform typed error |
| `.catch(f)` | Recover by returning another effect |
| `.catch_all(f)` | Recover with a fallback value and `Never` error |
| `.tap_error(f)` | Run an effect on failure and re-emit the error |
| `.provide_env(env)` | Capture a full environment and return `Effect<_, _, ()>` |
| `.zoom_env(f)` | Run an effect requiring `R` inside a larger environment `S` |
| `.local(f)` | Run with a cloned and modified local environment |
| `.ensuring(finalizer)` | Run finalizer after success or failure |
| `.on_exit(f)` | Observe `Exit` without changing result |

## Running Effects

| Function | Notes |
|----------|-------|
| `run_blocking(effect, env)` | Blocking synchronous runner |
| `run_async(effect, env).await` | Async runner |
| `#[effect_test]` | Attribute for tests that return `Effect`; harness executes and panics on `Err(E: Debug)` |
| `expect_effect_test(effect).await` | Async test helper for `R: Default`, panics on failure |
| `expect_effect_test_with_env(effect, env).await` | Async test helper with explicit environment |
| `expect_effect_test_with_layer(effect, layer).await` | Async test helper for `ServiceContext` plus `Layer` |
| `run_effect_test(effect).await` | Async test helper returning `Result<A, E>` |
| `run_effect_test_with_env(effect, env).await` | Async test helper returning `Result<A, E>` with explicit environment |
| `TestRuntime::with_env(fixture)` | Reusable async test adapter with environment fixture |
| `run_test(effect, env)` | Test runner returning `Exit<A, E>` |
| `run_test_with_clock(effect, env, clock)` | Test runner with explicit `TestClock` argument |

## Services and Layers

| Item | Notes |
|------|-------|
| `#[derive(Service)]` | Make a cloneable struct available through `Effect::service::<S>()` |
| `Effect::service::<S>()` | Read service `S` from `R: ServiceLookup<S>` |
| `ServiceContext::empty().add(service)` | Build a derive-service environment |
| `Layer::succeed(service)` | Infallible service layer |
| `Layer::effect(name, make)` | Effectful service layer |
| `layer.merge(other)` | Build independent services into one context |
| `layer.provide(provider)` | Use provider services to build the layer, then hide provider output |
| `layer.provide_merge(provider)` | Provide dependencies and keep both outputs |
| `layer.memoized()` | Cache the first layer build result |

## Tagged Context Helpers

| Item | Notes |
|------|-------|
| `service_key!(pub struct Key);` | Declare a nominal key type |
| `Tagged<Key, V>` / `Service<Key, V>` | Keyed service cell |
| `service_env::<Key, _>(value)` | Build a one-cell `Context` |
| `ctx.get::<Key>()` | Read a tagged value from a typed context |
| `effect.provide_head(value)` | Provide the head tagged dependency |

## Concurrency

| Function/Method | Notes |
|-----------------|-------|
| `run_fork(&runtime, || (effect, env))` | Spawn a fiber |
| `handle.join()` | Await result as `Result<A, Cause<E>>` |
| `handle.await_exit()` | Await result as `Exit<A, E>` |
| `handle.interrupt()` | Interrupt the fiber |
| `handle.status()` | Inspect `Running`, `Succeeded`, `Failed`, or `Interrupted` |
| `fiber_all(handles)` | Join many `FiberHandle`s into `Effect<Vec<A>, Cause<E>, ()>` |
| `with_fiber_id(id, f)` | Run `f` under a fiber id |

## STM

| Function | Notes |
|----------|-------|
| `TRef::make(v)` | Transactionally allocate a cell: `Stm<TRef<T>, ()>` |
| `tref.read_stm()` | Read inside `Stm` |
| `tref.write_stm(v)` | Write inside `Stm` |
| `tref.update_stm(f)` | Update value, returning `()` |
| `tref.modify_stm(f)` | Compute `(output, next)` and write `next` |
| `Stm::succeed(a)` / `Stm::fail(e)` | Transactional success/failure |
| `Stm::retry()` | Retry the current transaction |
| `commit(stm)` / `atomically(stm)` | Lift `Stm<A, E>` into `Effect<A, E, R>` |
| `TQueue::bounded(n)` / `TQueue::unbounded()` | Transactional FIFO queues |
| `queue.offer(v)` / `queue.take()` | Enqueue or dequeue transactionally |
| `TMap::make()` / `map.get`, `set`, `delete` | Transactional map |
| `TSemaphore::make(n)` / `acquire`, `release` | Transactional permit counter |

## Scheduling

| Function | Notes |
|----------|-------|
| `Schedule::recurs(n)` | Allow `n` additional iterations |
| `Schedule::spaced(d)` | Fixed delay |
| `Schedule::exponential(base)` | Exponential backoff |
| `schedule.compose(other)` | Continue while both schedules continue; use max delay |
| `schedule.jittered()` | Apply deterministic jitter |
| `Schedule::recurs_while(pred)` | Continue while predicate holds |
| `Schedule::recurs_until(pred)` | Continue until predicate holds |
| `retry(|| make_effect(), schedule)` | Retry a one-shot effect factory |
| `repeat(|| make_effect(), schedule)` | Repeat a one-shot effect factory |

## Streams and Sinks

| Function | Notes |
|----------|-------|
| `Stream::from_iterable(iter)` | Stream from iterable values |
| `Stream::from_effect(effect)` | Stream from `Effect<Vec<A>, E, R>` |
| `stream.map`, `filter`, `take`, `take_while`, `drop_while` | Stream transformations |
| `stream.run_collect()` | Collect all elements into `Vec<A>` |
| `stream.run_fold(init, f)` | Fold elements |
| `stream.run_for_each(f)` | Run synchronous consumer |
| `stream.run_for_each_effect(f)` | Run effectful consumer |
| `Sink::collect()` | Sink collecting elements into `Vec` |
| `Sink::fold_left(init, f)` | Folding sink |
| `Sink::drain()` | Discarding sink |
| `sink.run(stream)` | Run a sink against a stream |

## Schema

| Function | Notes |
|----------|-------|
| `Unknown::{Null, Bool, I64, F64, String, Array, Object}` | Dynamic input values |
| `string()`, `i64()`, `f64()`, `bool_()` | Primitive schemas |
| `optional(s)`, `array(s)` | Container schemas |
| `tuple`, `tuple3`, `tuple4` | Tuple schemas |
| `struct_`, `struct3`, `struct4` | Struct schemas from field schemas |
| `union_`, `union_chain` | Union schemas |
| `filter(s, pred, msg)` / `refine(...)` | Validation refinements |
| `transform(s, decode, encode)` | Bidirectional transformation |
| `schema.decode(input)` | Decode typed wire input |
| `schema.decode_unknown(&unknown)` | Decode `Unknown` input |
| `ParseError::new(path, message)` | Single parse issue |
| `ParseErrors::one(err)` / `ParseErrors::new(vec)` | Aggregate parse issues |
