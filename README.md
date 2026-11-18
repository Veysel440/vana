# vana

vana is an overload-protection proxy for PHP applications, written in Rust.
It is meant to sit in front of a PHP application and decide which requests
reach PHP, in what order, and how many at a time.

## Status

Pre-alpha. vana does not run as a proxy yet. The decision library and the
simulator exist; the HTTP layer does not. The core can queue a request and
reject one whose deadline has passed, and the simulator shows it; no HTTP
response is sent yet.

| Part | State | Phase |
|---|---|---|
| Time, tickets, permits (`vana-core`) | built | 1 |
| Fixed limiter (`vana-core`) | built | 1 |
| Deterministic simulator with a fixed limit (`vana-sim`) | built | 1 |
| Bounded FIFO queue with deadline-aware admission (`vana-core` gate, `vana-sim` gate scenarios) | built | 1 |
| Priority classes, per-tenant Deficit Round Robin | planned | 1 |
| Rejection with 503 and `Retry-After` for requests that cannot meet their deadline | planned | 2 |
| HTTP/1.1 reverse proxy | planned | 2 |
| AIMD and Gradient2 limiters, PHP-FPM status limiter | planned | 3 |
| Shadow mode, Prometheus metrics, structured logs, graceful shutdown | planned | not yet assigned |
| "What-if" simulation from access logs | planned | not yet assigned |
| Lab and comparative measurements | planned | 4 |

## Planned features

- HTTP/1.1 reverse proxy.
- Concurrency limits on requests sent to PHP: AIMD and Gradient2, plus a
  limit driven by the PHP-FPM status page (the Fixed limit is built).
- Priority classes, with fair queuing between tenants inside each class
  (Deficit Round Robin).
- Early rejection (503 with `Retry-After`) of requests that cannot finish
  before their deadline, without sending them to PHP. The core already
  refuses to grant a permit to a request whose deadline has passed.
- "What-if" simulation from access logs.
- Shadow mode: decisions are made and recorded as metrics, but not enforced.
- Prometheus metrics, structured logs and graceful shutdown.

See [docs/DESIGN.md](docs/DESIGN.md) for the design, scope and non-goals.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
