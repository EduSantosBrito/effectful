use effectful::testing::effect_test;

#[effect_test]
fn effect_returning_test_compiles_without_tokio_import() -> effectful::Effect<(), &'static str, ()>
{
  effectful::Effect::new(|_| Ok(()))
}

fn main() {}
