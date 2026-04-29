//! Bridge a queue-backed [`effectful::channel::QueueChannel`] to Axum: one HTTP exchange is
//! `write(Request)` then `read()` for the response.
//!
//! [`QueueChannel`] clones outbound elements when offering to the queue; use
//! [`Bytes`] bodies (`http::Request<Bytes>` / `http::Response<Bytes>`). Use
//! [`exchange_into_response`] with Axum's streaming [`Body`] (body is buffered
//! before the round-trip).
//!
//! Queue reads surface [`QueueError`] and are mapped to [`StatusCode`] here.
//! ([`QueueChannel`] is used instead of full [`effectful::channel::Channel`] so the handle
//! stays [`Send`] inside Axum [`axum::handler::Handler`] futures.)

use axum::body::{Body, Bytes};
use axum::http::{Request, Response, StatusCode};
use axum::response::{IntoResponse, Response as AxumResponse};
use effectful::channel::QueueChannel;
use effectful::{Effect, QueueError};
use http_body_util::BodyExt;

#[inline]
fn map_queue_error(e: QueueError) -> StatusCode {
  match e {
    QueueError::Disconnected => StatusCode::SERVICE_UNAVAILABLE,
  }
}

/// Build an effect that performs one round-trip on `ch`: enqueue `req`, then await the mapped
/// response. Failures map to `E` via [`From<StatusCode>`] (including empty read → `503`).
pub fn exchange<A, E, R>(
  ch: QueueChannel<Response<Bytes>, Request<Bytes>, R>,
  req: Request<Bytes>,
) -> Effect<A, E, R>
where
  A: From<Response<Bytes>> + 'static,
  E: From<StatusCode> + 'static,
  R: 'static,
{
  effectful_tower::queue_channel_exchange(ch, req)
    .map(|r: Response<Bytes>| r.into())
    .map_error(|e| map_queue_error(e).into())
}

/// Buffer `req`'s body to [`Bytes`], run [`exchange`], then map the wire response to Axum's
/// [`Body`].
#[inline]
pub async fn exchange_into_response<R: Send + 'static>(
  env: R,
  ch: QueueChannel<Response<Bytes>, Request<Bytes>, R>,
  req: Request<Body>,
) -> AxumResponse {
  let (parts, body) = req.into_parts();
  let bytes = match BodyExt::collect(body).await {
    Ok(c) => c.to_bytes(),
    Err(_) => return StatusCode::BAD_REQUEST.into_response(),
  };
  let req_b = Request::from_parts(parts, bytes);
  match effectful_tokio::run_effect_from_state_with(
    env,
    effectful_tokio::EffectExecution::Plain,
    move |_e| exchange::<Response<Bytes>, StatusCode, R>(ch, req_b),
  )
  .await
  {
    Ok(r) => {
      let (parts, b) = r.into_parts();
      AxumResponse::from_parts(parts, Body::from(b))
    }
    Err(s) => s.into_response(),
  }
}
