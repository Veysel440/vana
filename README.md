# vana

vana is an overload-protection proxy for PHP applications, written in Rust.
It is meant to sit in front of a PHP application and decide which requests
reach PHP, in what order, and how many at a time.

## Status

Pre-alpha. Nothing works yet: the repository contains the workspace skeleton,
CI configuration and design notes.

## Planned features

- HTTP/1.1 reverse proxy.
- Concurrency limits on requests sent to PHP: Fixed, AIMD and Gradient2, plus a
  limit driven by the PHP-FPM status page.
- Priority classes, with fair queuing between tenants inside each class
  (Deficit Round Robin).
- Early rejection (503 with `Retry-After`) of requests that cannot finish
  before their deadline, without sending them to PHP.
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
