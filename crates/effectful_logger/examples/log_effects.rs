//! Extract [`EffectLogger`] once with `bind* EffectLogger`, then call its methods
//! as `bind* logger.level(…)` steps — each returns `Effect<(), EffectLoggerError, R>`.
//!
//! Run: `RUST_LOG=trace devenv shell -- cargo run -p logger --example log_effects`

use ::effectful::{Cons, Context, Effect, Nil, Service, effect, run_blocking};
use effectful_logger::{EffectLogKey, EffectLogger, EffectLoggerError};

type LogCtx = Context<Cons<Service<EffectLogKey, EffectLogger>, Nil>>;

fn build_ctx() -> LogCtx {
  Context::new(Cons(Service::<EffectLogKey, _>::new(EffectLogger), Nil))
}

fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("trace")),
    )
    .init();

  let prog: Effect<(), EffectLoggerError, LogCtx> = effect!(|_r: &mut LogCtx| {
    let logger = bind* EffectLogger;
    bind* logger.trace("trace step");
    bind* logger.debug("debug step");
    bind* logger.info("info step");
    bind* logger.warn("warn step");
    bind* logger.error("error step");
  });

  run_blocking(prog, build_ctx()).expect("tracing never fails");
  println!("ran all five log levels via bind* EffectLogger");
}
