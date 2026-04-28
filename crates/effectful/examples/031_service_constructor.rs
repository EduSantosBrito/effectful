//! Ex 031 — a service value wraps itself in `ServiceContext`.
use effectful::{ContextService, Service};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Service)]
struct Port {
  value: u16,
}

fn main() {
  let context = Port { value: 8080 }.to_context();
  assert_eq!(context.get::<Port>().map(|port| port.value), Some(8080));
  println!("031_service_constructor ok");
}
