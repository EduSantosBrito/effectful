//! Mix bind* effect steps with ordinary Rust control flow inside `effect!`.
//! Extract the logger once, then use it inside an `if` block — demonstrating
//! that bind* works anywhere as an expression.
//!
//! Run: `devenv shell -- cargo run -p logger --example macro_block_tail`

use ::effectful::{Cons, Context, Effect, Nil, Service, effect, run_blocking, succeed};
use effectful_logger::{EffectLogKey, EffectLogger, EffectLoggerError};

type LogCtx = Context<Cons<Service<EffectLogKey, EffectLogger>, Nil>>;

fn build_ctx() -> LogCtx {
  Context::new(Cons(Service::<EffectLogKey, _>::new(EffectLogger), Nil))
}

fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::new("info"))
    .init();

  let program: Effect<i32, EffectLoggerError, LogCtx> = effect!(|_r: &mut LogCtx| {
    let logger = bind* EffectLogger;
    let seed = bind* succeed::<i32, EffectLoggerError, LogCtx>(6);
    if seed > 0 {
      bind* logger.info("seed is positive");
    }
    seed * 7
  });

  let out = run_blocking(program, build_ctx()).expect("tracing never fails");
  println!("block tail value: {out}");
}
