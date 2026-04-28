//! Ex 030 — `#[derive(Service)]` makes the service type its own key.
use effectful::Service;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Service)]
struct DbSvc {
  port: u32,
}

fn main() {
  let context = DbSvc { port: 7 }.to_context();
  assert_eq!(context.get::<DbSvc>().map(|svc| svc.port), Some(7));
  println!("030_service_def ok");
}
