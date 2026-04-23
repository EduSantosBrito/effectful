# Backpressure Policies — Controlling Flow

Channel-backed streams use `BackpressurePolicy` to decide what happens when the internal queue is full.

## The Problem

```text
Producer: emits 10,000 events/sec
Consumer: processes 1,000 events/sec

What happens to the 9,000 surplus events per second?
```

The answer is domain-specific: block, drop, or fail.

## BackpressurePolicy

```rust,ignore
use effectful::BackpressurePolicy;

BackpressurePolicy::BoundedBlock // wait until space is available
BackpressurePolicy::DropNewest   // discard the newly offered item
BackpressurePolicy::DropOldest   // evict the oldest queued item
BackpressurePolicy::Fail         // fail the producer enqueue effect
```

`backpressure_decision(policy, queue_len, capacity)` exposes the decision logic directly for tests or diagnostics.

## Channel-Backed Streams

```rust,ignore
use effectful::{BackpressurePolicy, Chunk, stream_from_channel_with_policy, send_chunk, end_stream};

let (stream, sender) = stream_from_channel_with_policy::<Event, AppError, ()>(
    1024,
    BackpressurePolicy::DropOldest,
);

run_blocking(send_chunk(&sender, Chunk::singleton(event)), ())?;
run_blocking(end_stream(sender), ())?;
```

Use `stream_from_channel(capacity)` for the default `BoundedBlock` policy.

## Choosing a Policy

| Scenario | Policy |
|----------|--------|
| No data loss acceptable | `BoundedBlock` |
| Latest value matters most | `DropOldest` |
| New overload data is expendable | `DropNewest` |
| Caller must know about overflow | `Fail` |

## Monitoring Drops

There is no built-in dropped-counter helper. If drops matter operationally, wrap `send_chunk` or your producer logic and record metrics at that boundary.

## Summary

Choose a policy explicitly. `BoundedBlock` is safest for correctness but can stall producers. `DropOldest` and `DropNewest` trade completeness for bounded memory. `Fail` surfaces overload as an error.
