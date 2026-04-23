//! Build an [`effectful_reqwest::Client`] with [`effectful_reqwest::layer_reqwest_client_default`], wrap it in a
//! [`effectful::Context`], and run an HTTP [`effectful::Effect`].
//!
//! Run: `cargo run -p effectful_reqwest --example 030_layer_default_env`

use effectful::Layer;
use effectful::context::{Cons, Context, Nil};
use effectful_reqwest::{Error, layer_reqwest_client_default, text};
use effectful_tokio::run_async;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::main]
async fn main() {
  let server = MockServer::start().await;
  Mock::given(method("GET"))
    .and(path("/layer"))
    .respond_with(ResponseTemplate::new(200).set_body_string("layer-ok"))
    .mount(&server)
    .await;

  let url = format!("{}/layer", server.uri());
  let layer = layer_reqwest_client_default();
  let cell = layer.build().unwrap();
  let env: Context<Cons<_, Nil>> = Context::new(Cons(cell, Nil));

  let body = run_async(text::<String, Error, _, _>(move |c| c.get(url)), env)
    .await
    .unwrap();
  assert_eq!(body, "layer-ok");
  println!("030_layer_default_env ok");
}
