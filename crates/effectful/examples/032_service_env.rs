//! Ex 032 — `ServiceContext` is the v1 service environment.
use effectful::{Effect, MissingService, Service, ServiceContext, run_blocking};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Token {
  value: &'static str,
}

fn main() {
  let env = Token { value: "secret" }.to_context();
  assert_eq!(env.get::<Token>().map(|token| token.value), Some("secret"));

  let program: Effect<&'static str, MissingService, ServiceContext> =
    Token::use_sync(|token| token.value);
  assert_eq!(run_blocking(program, env), Ok("secret"));
  println!("032_service_env ok");
}
