//! Log inside `effect!` using `bind* EffectLogger` extraction and `bind* logger.level(…)` steps,
//! then compute a return value — demonstrating the full service/tag pattern.
//!
//! Run: `devenv shell -- cargo run -p logger --example macro_sync_bind`

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

  // Extract the logger once, then use it across multiple steps.
  // bind* succeed(...) binds a pure value; bind* logger.info(...) logs as an effect step.
  let program: Effect<i32, EffectLoggerError, LogCtx> = effect!(|_r: &mut LogCtx| {
    let logger = bind* EffectLogger;
    bind* logger.info("first step: log_info via bind* logger");
    bind* logger.warn("second step: log_warn via bind* logger");
    let n = bind* succeed::<i32, EffectLoggerError, LogCtx>(21);
    let m = bind* succeed::<i32, EffectLoggerError, LogCtx>(n * 2);
    m
  });

  let out = run_blocking(program, build_ctx()).expect("tracing never fails");
  println!("effect! tail value: {out}");
}
