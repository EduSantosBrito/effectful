# Mocking Services — Test Doubles via Layers

In effectful, a test double is just a different service value or layer. Production code gets a real service; tests get an in-memory or spy service with the same service type.

## The Pattern

Wrap your interface in a cloneable service struct.

```rust,ignore
trait DbImpl: Send + Sync {
    fn get_user(&self, id: UserId) -> Effect<User, DbError, ()>;
    fn save_user(&self, user: User) -> Effect<(), DbError, ()>;
}

#[derive(Clone, Service)]
struct Db {
    inner: Arc<dyn DbImpl>,
}

impl Db {
    fn get_user(&self, id: UserId) -> Effect<User, DbError, ()> {
        self.inner.get_user(id)
    }

    fn save_user(&self, user: User) -> Effect<(), DbError, ()> {
        self.inner.save_user(user)
    }
}
```

## Test Double

```rust,ignore
struct InMemoryDb {
    users: Mutex<HashMap<UserId, User>>,
}

impl DbImpl for InMemoryDb {
    fn get_user(&self, id: UserId) -> Effect<User, DbError, ()> {
        match self.users.lock().expect("users lock").get(&id).cloned() {
            Some(user) => succeed(user),
            None => fail(DbError::NotFound(id)),
        }
    }

    fn save_user(&self, user: User) -> Effect<(), DbError, ()> {
        self.users.lock().expect("users lock").insert(user.id, user);
        succeed(())
    }
}
```

## Injecting the Test Double

```rust,ignore
#[effect_test(env = "test_env")]
fn get_user_returns_saved_user() -> Effect<(), DbError, ServiceContext> {
    let effect = Db::use_(|db| {
        effect! {
            bind* db.save_user(User { id: UserId::new(1), name: "Alice".into() });
            let user = bind* db.get_user(UserId::new(1));
            assert_eq!(user.name, "Alice");
        }
    });

    effect
}

fn test_env() -> ServiceContext {
    let db = Db { inner: Arc::new(InMemoryDb::new()) };
    db.to_context()
}
```

Business logic is unchanged. Only the service value changes.

## Spies

When you need to assert calls, add tracking to the test double.

```rust,ignore
#[derive(Clone, Service)]
struct Mailer {
    sent: Arc<Mutex<Vec<Email>>>,
}

impl Mailer {
    fn send(&self, email: Email) -> Effect<(), MailError, ()> {
        self.sent.lock().expect("sent lock").push(email);
        succeed(())
    }
}

#[test]
fn registration_sends_welcome_email() {
    let mailer = Mailer { sent: Arc::new(Mutex::new(Vec::new())) };
    let env = mailer.clone().to_context();

    let exit = run_test(register_user("alice@example.com"), env);
    assert!(matches!(exit, Exit::Success(_)));

    let sent = mailer.sent.lock().expect("sent lock");
    assert_eq!(sent.len(), 1);
}
```

## Failing Services

Test failure handling by providing a service whose methods fail.

```rust,ignore
struct FailingDb;

impl DbImpl for FailingDb {
    fn get_user(&self, _id: UserId) -> Effect<User, DbError, ()> {
        fail(DbError::ConnectionLost)
    }

    fn save_user(&self, _user: User) -> Effect<(), DbError, ()> {
        fail(DbError::ConnectionLost)
    }
}

#[test]
fn get_user_propagates_db_errors() {
    let env = Db { inner: Arc::new(FailingDb) }.to_context();
    let exit = run_test(get_user(UserId::new(1)), env);
    assert!(matches!(exit, Exit::Failure(Cause::Fail(DbError::ConnectionLost))));
}
```

## Layer-Based Setup

For larger tests, package doubles in layers.

```rust,ignore
fn test_layer() -> Layer<(Db, Mailer), AppError, ()> {
    Layer::succeed(Db { inner: Arc::new(InMemoryDb::new()) })
        .merge(Layer::succeed(Mailer::spy()))
}

#[effect_test(layer = "test_layer")]
fn full_registration_flow_works() -> Effect<(), AppError, ServiceContext> {
    full_registration_flow().void()
}
```

## What You Don't Need

- No mock framework.
- No `#[cfg(test)]` in business logic.
- No global service registry reset between tests.
- No special mocking API beyond ordinary services and layers.
