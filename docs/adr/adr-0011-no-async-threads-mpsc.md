# ADR-0011: No Async Runtime — Blocking Threads + mpsc Channel

- Status: accepted
- Date: 2026-03-23

## Context

The application performs long-running I/O: fetching project lists from
maps.lizaalert.ru, downloading map archives, and reading large files from disk. These
operations must not block the UI thread.

Two concurrency models were available:
- **async/await + runtime (Tokio/async-std)** — non-blocking, efficient, standard
  for Rust network code
- **`std::thread::spawn` + `std::sync::mpsc`** — blocking threads, simple mental model

egui/eframe does not use an async runtime internally. Adding Tokio alongside an
immediate-mode UI introduces a two-runtime question and async-Rust complexity that
brings little benefit for the current scope.

## Decision

Use **`std::thread::spawn`** for all long-running operations with
**`std::sync::mpsc::channel`** for communication back to the UI thread.

Pattern:
1. UI action triggers `AppState` method
2. Method clones what the thread needs, spawns a blocking thread
3. Thread does its work and sends `BackgroundMessage` variants over the `Sender`
4. `AppState::poll_background_tasks()` drains the `Receiver` each frame and updates
   application state

`reqwest` is used in blocking mode only (`features = ["blocking"]`). No `.await`,
no `async fn`, no Tokio anywhere in the codebase.

## Consequences

### Positive

- No async Rust complexity: no pinning, no executor, no `Send + 'static` bound fights
- Background work is easy to reason about and test
- No runtime dependency; binary is smaller
- egui's frame loop and application state live on one thread with no cross-thread
  contention

### Negative

- Cannot spawn many concurrent tasks cheaply; each operation gets a dedicated OS thread
- Parallelism is limited: downloading two maps at once requires spawning two threads
  manually
- If the domain ever needs truly concurrent operations (real-time GPS streaming,
  peer sync), this model becomes a bottleneck
- `mpsc` channel is one-consumer: all background messages funnel through a single
  `AppState` drain point

## Rejected Alternatives

### Tokio + async reqwest

Rejected because it would add ~500 KB to the binary, require `#[tokio::main]` or
a manual runtime block in `main.rs`, and complicate the boundary between the egui
frame loop and async task scheduling for no concrete gain at current scope.

## Follow-Up

If Phase 7 or later requires concurrent operations (e.g., streaming live GPS tracks
while a map download is running), revisit this decision. Adding `rayon` for CPU-bound
work (tile decoding, GPX parsing) is compatible with this model without adding an
async runtime.
