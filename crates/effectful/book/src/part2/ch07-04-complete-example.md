# A Complete DI Example — Putting It Together

This example uses the current derive-service / `ServiceContext` layer API.

## Domain

```rust,ignore
struct User { id: u64, name: String, email: String }
struct Post { id: u64, author_id: u64, title: String, body: String }
enum AppError { Db(DbError), Notify(NotifyError), Missing(MissingService) }
```

## Services

```rust,ignore
#[derive(Clone, Service)]
struct UserRepository {
    inner: Arc<dyn UserRepositoryImpl>,
}

#[derive(Clone, Service)]
struct PostRepository {
    inner: Arc<dyn PostRepositoryImpl>,
}

#[derive(Clone, Service)]
struct Notifier {
    inner: Arc<dyn NotificationImpl>,
}
```

Each service struct is cloneable and acts as its own lookup key.

## Business Logic

```rust,ignore
fn get_author_feed(author_id: u64) -> Effect<(User, Vec<Post>), AppError, ServiceContext> {
    UserRepository::use_(move |users| {
        PostRepository::use_(move |posts| {
            effect! {
                let user = bind* users.get_user(author_id).map_error(AppError::Db);
                let posts = bind* posts.get_posts_by_author(author_id).map_error(AppError::Db);
                (user, posts)
            }
        })
    })
}
```

The function depends on service types, not concrete infrastructure.

## Production Layers

```rust,ignore
let config_layer = Layer::succeed(Config::from_env());

let db_layer = Layer::effect("Database", || {
    Config::use_(|config| Database::connect(config.database_url))
});

let user_repo_layer = Layer::effect("UserRepository", || {
    Database::use_(|db| succeed(UserRepository::postgres(db)))
});

let post_repo_layer = Layer::effect("PostRepository", || {
    Database::use_(|db| succeed(PostRepository::postgres(db)))
});

let app_layer = user_repo_layer
    .merge(post_repo_layer)
    .provide_merge(db_layer.provide_merge(config_layer));
```

Run at the edge:

```rust,ignore
fn main() {
    let result = run_blocking(get_author_feed(1).provide(app_layer), ());
    println!("{result:?}");
}
```

## Test Wiring

```rust,ignore
fn test_layer() -> Layer<(UserRepository, PostRepository), AppError, ()> {
    Layer::succeed(UserRepository::in_memory([alice(), bob()]))
        .merge(Layer::succeed(PostRepository::in_memory([alice_post()])))
}

#[test]
fn feed_includes_author_posts() {
    let exit = run_test(get_author_feed(1).provide(test_layer()), ());
    assert!(matches!(exit, Exit::Success((_, posts)) if posts.len() == 1));
}
```

## What This Demonstrates

Business logic is decoupled from infrastructure:

- It reads services from `ServiceContext`.
- Production and tests provide different layers.
- Layer composition happens at the edge.
- Missing services fail through `MissingService` if requested at runtime.

Part III shifts to operational concerns: errors, fibers, resources, and scheduling.
