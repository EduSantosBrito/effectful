# Migrating from `async fn` to effects

This appendix shows the current migration shape for effectful: return `Effect`, keep dependencies in `R`, and run at the boundary with an explicit environment.

## Mental Model Shift

In ordinary async Rust, calling an `async fn` creates a `Future`; awaiting it runs the work.

```rust,ignore
async fn get_user(id: u64, db: &DbClient) -> Result<User, DbError> {
    db.query_one(id).await
}
```

In effectful, a function returns an `Effect` description. The runner receives the environment later.

```rust,ignore
fn get_user(id: u64) -> Effect<User, DbError, DbClient> {
    effect!(|db: &mut DbClient| {
        bind* db.query_one(id)
    })
}

let user = run_blocking(get_user(42), db_client)?;
```

## Pattern 1: async fn to Effect

**Before**

```rust,ignore
pub async fn process_order(
    order_id: OrderId,
    db: &DbClient,
    mailer: &MailClient,
) -> Result<Receipt, AppError> {
    let order = db.get_order(order_id).await?;
    let receipt = db.complete_order(order).await?;
    mailer.send_receipt(&receipt).await?;
    Ok(receipt)
}
```

**After**

```rust,ignore
#[derive(Clone)]
struct AppEnv {
    db: DbClient,
    mailer: MailClient,
}

pub fn process_order(order_id: OrderId) -> Effect<Receipt, AppError, AppEnv> {
    effect!(|env: &mut AppEnv| {
        let order = bind* env.db.get_order(order_id).map_error(AppError::Db);
        let receipt = bind* env.db.complete_order(order).map_error(AppError::Db);
        bind* env.mailer.send_receipt(&receipt).map_error(AppError::Mail);
        receipt
    })
}
```

Migration steps:

1. Change `async fn` to `fn` returning `Effect<A, E, R>`.
2. Move dependencies into an environment type or service context.
3. Replace `.await?` on effectful operations with `bind*`.
4. Return the success value as the block tail.
5. Call `run_blocking(effect, env)` or `run_async(effect, env)` at the boundary.

## Pattern 2: Wrapping Third-Party Async

Third-party libraries return futures, not effects. Wrap them with `from_async`.

```rust,ignore
fn fetch_price(symbol: String) -> Effect<f64, reqwest::Error, ()> {
    from_async(move |_r: &mut ()| async move {
        let response = reqwest::get(format!("https://api.example.com/price/{symbol}"))
            .await?;
        let body = response.json::<PriceResponse>().await?;
        Ok(body.price)
    })
}
```

Inside the async closure, use normal `.await`. Outside, compose the result as an `Effect`.

## Pattern 3: Error Types

Map infrastructure errors into your application error at composition points.

```rust,ignore
#[derive(Debug)]
enum AppError {
    Db(DbError),
    Mail(MailError),
}

let effect = db_call().map_error(AppError::Db);
```

Use `catch` to recover with another effect, and `catch_all` to turn typed errors into fallback success values.

## Pattern 4: Services

For application dependency injection, prefer `#[derive(Service)]` plus `ServiceContext`.

```rust,ignore
#[derive(Clone, Service)]
struct AppState {
    request_count: Arc<AtomicU64>,
}

fn handler() -> Effect<Response, AppError, ServiceContext> {
    AppState::use_sync(|state| {
        state.request_count.fetch_add(1, Ordering::Relaxed);
        Response::ok()
    })
}

let env = AppState::new().to_context();
let response = run_blocking(handler(), env)?;
```

For tagged HList contexts, use `service_key!(pub struct Key);`, `service_env::<Key, _>(value)`, and `Context::get::<Key>()`.

## Pattern 5: Transactional State

Use `TRef` when state updates must compose transactionally.

```rust,ignore
let counter = run_blocking(commit(TRef::make(0_u64)), ())?;

fn increment(counter: TRef<u64>) -> Effect<u64, (), ()> {
    effect! {
        bind* commit(counter.update_stm(|n| n + 1));
        bind* commit(counter.read_stm())
    }
}
```

There is no `stm!` macro in the current API; compose transactions with `flat_map`, `map`, and helpers like `update_stm`.

## Pattern 6: Resource Cleanup

Use `Scope` when cleanup must run at an explicit lifetime boundary.

```rust,ignore
fn with_connection<A, E, F>(pool: Pool<Connection, DbError>, f: F) -> Effect<A, E, ()>
where
    F: FnOnce(Connection) -> Effect<A, E, ()> + 'static,
    A: 'static,
    E: From<DbError> + 'static,
{
    scope_with(move |scope| {
        effect! {
            let conn = bind* pool.get().provide_env(scope.clone()).map_error(E::from);
            let close_conn = conn.clone();
            scope.add_finalizer(Box::new(move |_| close_conn.close()));
            bind* f(conn)
        }
    })
}
```

For pooled resources, `Pool::get()` registers return-to-pool cleanup on the provided `Scope`.

## Migration Strategy

1. Convert leaf async wrappers first with `from_async`.
2. Introduce explicit environment structs or `ServiceContext`.
3. Move `run_blocking` / `run_async` to program edges.
4. Convert tests to pass test environments or test layers.
5. Replace stale helper assumptions with current names: `run_collect`, `run_fold`, `retry(|| ..., schedule)`, `TRef::make`, `run_test(effect, env)`.

You can mix old async code with effects during migration. Wrap async futures at the edge and keep new domain workflows as `Effect` values.
