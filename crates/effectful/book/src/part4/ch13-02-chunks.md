# Chunks — Batched Stream Data

`Chunk<A>` is the batch container used by stream internals. It wraps a `Vec<A>` and makes chunk-level stream semantics explicit.

## What Is a Chunk

```rust,ignore
use effectful::Chunk;

let chunk = Chunk::from_vec(vec![1, 2, 3, 4, 5]);

assert_eq!(chunk.len(), 5);
assert!(!chunk.is_empty());

for item in chunk.iter() {
    println!("{item}");
}
```

## Why Chunks Exist

Streams pull values in batches. Operators can transform a whole `Chunk` internally instead of paying per-element overhead at every boundary.

```text
Single-element model:
  elem1 -> map -> filter -> emit -> elem2 -> map -> filter -> emit -> ...

Chunk model:
  chunk[1..64] -> map chunk -> filter chunk -> emit chunk -> ...
```

## Working with Chunks

```rust,ignore
let doubled = chunk.map(|x| x * 2);
let values = doubled.into_vec();
```

The current public `Chunk` API is intentionally small:

| Method | Purpose |
|--------|---------|
| `Chunk::empty()` | Empty chunk |
| `Chunk::singleton(value)` | One-element chunk |
| `Chunk::from_vec(values)` | Build from a vector |
| `.len()` / `.is_empty()` | Inspect size |
| `.iter()` | Iterate by reference |
| `.into_vec()` | Consume into `Vec<A>` |
| `.map(f)` | Transform elements |
| `.sort_with(order)` / `.compare_by(other, order)` | Runtime ordering helpers |

## In Streams and Sinks

Most application code does not construct chunks directly. `Stream::poll_next_chunk` and sink drivers use chunks internally; user-facing APIs usually expose element-wise transformations or final `Effect` consumers.
