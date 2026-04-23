# Service Traits — Defining Interfaces

Services can be modeled as cloneable structs that contain concrete clients, trait objects, or function handles. Derive `Service` on the struct to make it available through `ServiceContext`.

## Define the Interface

```rust,ignore
trait UserRepositoryImpl: Send + Sync {
    fn get_user(&self, id: u64) -> Effect<User, DbError, ()>;
    fn save_user(&self, user: User) -> Effect<(), DbError, ()>;
}

#[derive(Clone, Service)]
struct UserRepository {
    inner: Arc<dyn UserRepositoryImpl>,
}

impl UserRepository {
    fn get_user(&self, id: u64) -> Effect<User, DbError, ()> {
        self.inner.get_user(id)
    }
}
```

The service struct is the lookup key. It must be `Clone + 'static`.

## Accessing a Service

Use `Effect::service::<S>()`, `S::use_`, or `S::use_sync`.

```rust,ignore
fn get_user_profile(id: u64) -> Effect<UserProfile, AppError, ServiceContext> {
    UserRepository::use_(move |repo| {
        repo.get_user(id)
            .map(UserProfile::from)
            .map_error(AppError::Db)
    })
}
```

If the service is missing, the lookup fails with `MissingService`; include `From<MissingService>` in your application error when using `use_` / `Effect::service`.

## Tagged Alternative

The older typed-context style uses `service_key!(pub struct Key);` plus `Tagged<Key, V>` / `Service<Key, V>`.

```rust,ignore
service_key!(pub struct UserRepositoryKey);
type UserRepositoryService = Service<UserRepositoryKey, Arc<dyn UserRepositoryImpl>>;
```

Use this when you specifically want HList `Context` types in function signatures. Prefer derive-service for application service tables.

## Keeping Services Focused

Avoid a single god service. Split by capability.

```rust,ignore
#[derive(Clone, Service)]
struct Users { /* ... */ }

#[derive(Clone, Service)]
struct Mailer { /* ... */ }

#[derive(Clone, Service)]
struct Payments { /* ... */ }
```

Functions should read exactly the services they need from `ServiceContext`.
