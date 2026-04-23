# Glossary

Key terms used throughout this book.

---

**`bind*` (bind operator)**
The prefix operator inside `effect!` that runs an inner effect and yields its success value. `let x = bind* eff` binds the value; `bind* eff;` runs and discards it.

---

**Backpressure**
The mechanism by which a slow consumer controls a fast producer. In effectful streams this is represented by `BackpressurePolicy`: `BoundedBlock`, `DropNewest`, `DropOldest`, or `Fail`.

---

**Brand**
A zero-cost nominal wrapper. `Brand<String, EmailMarker>` and `Brand<String, NameMarker>` are different types even if both wrap `String`.

---

**`Cause<E>`**
The structured reason an effect failed: `Fail(E)`, `Die(String)`, `Interrupt(FiberId)`, or composed causes `Both` / `Then`.

---

**`Chunk<A>`**
A batch of stream elements. Streams pull and process chunks rather than one element at a time internally.

---

**Clock**
A trait abstracting time. `LiveClock` uses real time; `TestClock` is controlled by tests.

---

**`commit` / `atomically`**
Functions that lift `Stm<A, E>` into `Effect<A, E, R>`. Running the effect executes the transaction and retries on conflicts or `Stm::retry()`.

---

**`Context<...>`**
Typed heterogeneous environment for tagged services, built from `Cons` / `Nil` cells containing `Tagged<K, V>` values.

---

**`Effect<A, E, R>`**
The central type: a lazy description of work that succeeds with `A`, fails with `E`, and requires environment `R`.

---

**`effect!` macro**
Do-notation for `Effect`. It rewrites `bind* effect` into bind/await plumbing and wraps the block tail as success.

---

**`Exit<A, E>`**
Terminal outcome: `Exit::Success(A)` or `Exit::Failure(Cause<E>)`. Returned by `run_test` and `FiberHandle::await_exit()`.

---

**Fiber**
A lightweight unit of concurrent work. Spawn with `run_fork`; await with `join()` or `await_exit()`.

---

**`FiberRef`**
A fiber-scoped variable used for request ids, tracing context, and similar dynamic data.

---

**`from_async`**
Constructor that lifts an async closure returning `Result<A, E>` into an `Effect<A, E, R>`.

---

**HList**
The compile-time linked-list shape `Cons<Head, Tail>` / `Nil` used by the typed `Context` API.

---

**Layer**
A lazy recipe for building services into a `ServiceContext`. Compose layers with `merge`, `provide`, `provide_merge`, and `memoized()`.

---

**`Never`**
The uninhabited runtime error type (`Infallible`) used when an effect cannot fail through its typed error channel.

---

**`ParseError` / `ParseErrors`**
Schema parse failures. Build a single issue with `ParseError::new(path, message)` and aggregate with `ParseErrors::one` or `ParseErrors::new`.

---

**`R` (environment type parameter)**
The third parameter of `Effect<A, E, R>`. It encodes required dependencies in the type.

---

**`run_blocking`**
Synchronous runner: `run_blocking(effect, env)`. Use at application/test boundaries, not inside reusable library functions.

---

**`run_test`**
Test runner: `run_test(effect, env) -> Exit<A, E>`. It resets and checks the test leak counters around the run.

---

**Schedule**
A value describing delays and repeat limits for the free `retry(|| effect, schedule)` and `repeat(|| effect, schedule)` functions. Current constructors include `recurs`, `spaced`, `exponential`, `recurs_while`, and `recurs_until`.

---

**Schema**
A `Schema<A, I, E>` decodes wire value `I` or dynamic `Unknown` into `A` and can encode `A` back to `I`.

---

**Scope**
A resource lifetime boundary. Register finalizers with `Scope::add_finalizer`; close the scope to run finalizers.

---

**`service_key!`**
Macro for declaring a nominal key type, e.g. `service_key!(pub struct DbKey);`. Pair it with `Tagged<DbKey, Db>` / `Service<DbKey, Db>`.

---

**Sink**
A stream consumer represented by `Sink<Out, In, E, R>`. Built-ins include `Sink::collect`, `Sink::fold_left`, and `Sink::drain`; run with `sink.run(stream)`.

---

**`Stm<A, E>`**
A composable transactional computation. Use `Stm::succeed`, `Stm::fail`, `Stm::retry`, `flat_map`, and `map`; run with `commit` / `atomically`.

---

**Stream**
A pull stream of values with typed error and environment. Build with `from_iterable` or `from_effect`; consume with `run_collect`, `run_fold`, or sinks.

---

**Tag / Tagged**
The typed key/value mechanism for contexts. `Tagged<K, V>` stores a value `V` under nominal key `K`.

---

**`TestClock`**
Controllable clock for deterministic scheduling tests. Construct with `TestClock::new(start_instant)`.

---

**`TRef<T>`**
A transactional mutable cell. Allocate with `TRef::make(value)` inside `Stm`; use `read_stm`, `write_stm`, `update_stm`, and `modify_stm`.

---

**`Unknown`**
Dynamic input for schemas. Current variants are `Null`, `Bool`, `I64`, `F64`, `String`, `Array`, and `Object`.
