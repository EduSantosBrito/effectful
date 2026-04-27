use effectful::{Cons, Context, Nil, Tagged};

struct ConfigKey;
struct ClockKey;

fn read_clock<'a, R>(
  context: &'a Context<Cons<Tagged<ConfigKey, &'static str>, R>>,
) -> &'a <R as effectful::Get<ClockKey>>::Target
where
  R: effectful::Get<ClockKey, Target = u64>,
{
  context.get_after_head::<ClockKey>()
}

fn main() {
  let context = Context::new(Cons(
    Tagged::<ConfigKey, _>::new("test"),
    Cons(Tagged::<ClockKey, _>::new(42_u64), Nil),
  ));

  assert_eq!(*read_clock(&context), 42);
}
