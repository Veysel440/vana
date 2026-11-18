# vana design notes

Status: draft, written in Phase 0. Nothing described here is implemented yet.
Points that are still open are marked `TODO:`.

## 1. Problem

A PHP application served by PHP-FPM can only work on as many requests at once
as it has worker processes: each worker handles one request at a time, and
`pm.max_children` caps the size of the pool.

When requests arrive faster than the workers can finish them, the excess waits
in a queue in front of the pool. Under overload that queue grows, and latency
grows with it. Clients start to time out, work done for a client that has
already given up is wasted, and retries from clients and calling services add
load at the moment when there is no capacity left.

The wait itself may have no limit. According to the FrankenPHP documentation
for classic mode, queued connections wait indefinitely for a free PHP thread
unless `max_wait_time` is set.

vana's job is to decide, before a request reaches PHP, whether it goes now,
waits, or is turned away.

## 2. Thesis

1. **Adaptive limit.** The number of concurrent requests sent to the upstream
   is adjusted from observed latency instead of being fixed in advance.
2. **Priority and fairness.** Requests are sorted into priority classes.
   Inside a class, tenants are served fairly with Deficit Round Robin (DRR), so
   one busy tenant cannot take the whole share of its class.
3. **Early rejection.** A request that cannot finish before its deadline is
   rejected right away with 503 and `Retry-After`, and is never sent to PHP.

## 3. MVP scope

- HTTP/1.1 reverse proxy.
- Three or four priority classes. TODO: final number and names.
- Per-tenant DRR inside each class.
- Limiters: Fixed, AIMD (additive increase, multiplicative decrease) and
  Gradient2.
- Deadline-based early rejection: 503 with `Retry-After`.
- A limiter that reads the PHP-FPM status page, which has JSON output. Fields
  used: `listen queue`, `active processes`, `idle processes` and
  `max children reached`. TODO: how these fields are turned into a limit.
- "What-if" simulation from access logs: recorded traffic is replayed through
  the simulator to show what vana would have decided. TODO: supported log
  formats.
- Prometheus metrics.
- Structured logs.
- Graceful shutdown.
- Shadow mode: vana makes every decision but does not enforce it; decisions
  are only recorded as metrics.

## 4. Non-goals (MVP)

- TLS termination.
- HTTP/2 and HTTP/3.
- WebSocket and SSE beyond passthrough.
- Speaking FastCGI directly to PHP-FPM.
- Coordination between multiple vana instances.
- A web UI.

## 5. Later

- Laravel companion package: route priorities, and a header carrying the
  remaining time budget.
- Brownout/degrade header.
- Waiting-room mode.
- Retry-storm protection.
- PostgreSQL front end: a separate product built on the same core.

## 6. Architecture

```text
vana  --->  vana-core  <---  vana-sim
```

- `vana-core` (library): the decision engine. No I/O, no async runtime, no
  dependencies.
- `vana` (binary): the HTTP proxy. It owns sockets, timers, configuration,
  metrics and logs, and asks `vana-core` what to do.
- `vana-sim` (library): a simulator that drives `vana-core` on virtual time.

Dependencies point only towards `vana-core`. Because the core performs no I/O
and reads time only through a clock supplied by its caller, the proxy and the
simulator run the same decision code.

Request flow:

```text
classify -> admit -> queue -> limit -> upstream -> feedback
```

1. **classify**: find the request's priority class, tenant and deadline.
   TODO: where the deadline comes from (a per-class default, a request header,
   or both).
2. **admit**: decide whether to accept the request at all. A request whose
   deadline cannot be met is rejected here with 503 and `Retry-After`.
   TODO: how the expected wait is estimated.
3. **queue**: an admitted request waits in the scheduler.
4. **limit**: the scheduler releases the next request only when the limiter
   grants a permit. The deadline is checked again at this point, so an expired
   request is never sent (I3). TODO: the response an expired request gets.
5. **upstream**: the request is forwarded to the PHP application.
6. **feedback**: when the upstream exchange ends, the permit is released: the
   slot is freed and a latency sample goes to the limiter.

## 7. Core concepts

- **Ticket**: a request as the core sees it: class, tenant, arrival time and
  deadline. A ticket ends in exactly one way: sent upstream, rejected, expired,
  or cancelled because the client went away.
- **Permit**: the right to have one request in flight upstream. It is an RAII
  guard: dropping it frees the slot and records a latency sample. Because
  release is tied to drop, the slot also comes back on errors and client
  disconnects, not only on success. TODO: how the limiter treats samples from
  errors and client disconnects.
- **Scheduler**: priority classes, DRR between the tenants inside each class,
  and aging so that low-priority requests are not starved. TODO: the cost unit
  used by DRR (one per request, or weighted).
- **Limiter**: a trait that answers "how many requests may be in flight now".
  Planned implementations: Fixed, AIMD, Gradient2 and the PHP-FPM status
  limiter.
- **Clock**: the core's only source of time. The proxy uses the real clock;
  tests and the simulator use virtual time.

## 8. Invariants (to be tested in Phase 1)

These apply when vana enforces its decisions; in shadow mode the decisions are
only recorded.

- **I1**: The number of concurrent requests at the upstream never exceeds the
  current limit. TODO: define the case where the limit drops below the number
  of requests already in flight.
- **I2**: For every admitted ticket that receives a permit, the permit is
  released exactly once, including when the client disconnects; a ticket that
  ends before it receives a permit never receives one.
- **I3**: A request whose deadline has passed is never sent upstream.
- **I4**: Within a class, every waiting tenant is served within a bounded
  number of rounds; no tenant starves.
- **I5**: Thanks to aging, a low-priority request does not wait without bound.
  TODO: the value of the bound, to be defined in Phase 1.
- **I6**: The limit always stays within `[min, max]`.
- **I7**: Given the same seed and the same input, a simulation run produces the
  same result.

## 9. Testing strategy

Correctness and performance are tested separately.

Correctness:

- Property-based tests of `vana-core`: random sequences of arrivals,
  completions, cancellations and clock steps, checked against the invariants
  above. TODO: whether `vana-core`, which has no dependencies, may take a
  test-only dependency for this, or produces its random inputs itself.
- Deterministic simulation in `vana-sim` on virtual time. A run is fully
  determined by its seed and input (I7), so a failing run can be replayed
  exactly.
- Integration tests of the `vana` binary against a real upstream.

Performance:

- Load comes from a separate machine, never from the machine under test.
- Open workload model: requests arrive at a fixed rate, independent of how
  quickly responses come back.
- Raw data is committed to the repository, together with the hardware and
  software versions used.
- Results are never invented or estimated. A number that was not measured is
  not reported.

## 10. Prior art

- [Envoy adaptive concurrency filter](https://www.envoyproxy.io/docs/envoy/latest/configuration/http/http_filters/adaptive_concurrency_filter):
  adjusts the concurrency limit from measured latency; its documentation
  describes no request queue and no priorities.
- [HAProxy overload protection](https://www.haproxy.com/documentation/haproxy-configuration-tutorials/performance/overload-protection/):
  a fixed `maxconn` set in the configuration, 503 once a queued request reaches
  `timeout queue`, and `set-priority-class` since HAProxy 1.9.
- [FrankenPHP classic mode](https://frankenphp.dev/docs/classic/):
  `max_wait_time` limits how long a request waits for a free PHP thread.
- [Netflix, "Performance Under Load"](https://netflixtechblog.medium.com/performance-under-load-3e6fa9a60581)
  and [Netflix/concurrency-limits](https://github.com/Netflix/concurrency-limits):
  the article on adaptive concurrency limits at Netflix, and the library that
  implements them.
- [lua-resty-adaptive-limit](https://github.com/mrjavadseydi/lua-resty-adaptive-limit):
  an adaptive concurrency limit for OpenResty; according to its README, it has
  no fairness between tenants inside a pool.
- [Facebook, "Fail at Scale"](https://queue.acm.org/detail.cfm?id=2839461):
  describes CoDel and adaptive LIFO as defenses against resource exhaustion.
- [Uber Cinnamon](https://www.uber.com/us/en/blog/cinnamon-using-century-old-tech-to-build-a-mean-load-shedder/):
  Uber's priority-based load shedder.
- [Uber, "How Uber Conquered Database Overload"](https://www.uber.com/tr/en/blog/from-static-rate-limiting-to-intelligent-load-management/):
  load management for Uber's own databases (2026), with CoDel and adaptive
  LIFO, priority-based load shedding and per-tenant limits.
- [PHP-FPM status page](https://www.php.net/manual/en/fpm.status.php):
  the fields read by the planned PHP-FPM limiter, and the page's JSON output.

## 11. Milestones

- **Phase 0**: skeleton, CI and these design notes.
- **Phase 1**: core and simulator; the invariants above are tested.
- **Phase 2**: HTTP proxy.
- **Phase 3**: adaptive limits and the PHP-FPM signal.
- **Phase 4**: the lab and comparative measurements.
- **Phase 5**: release.

## 12. Open questions

- Is the name available?
- How is a tenant identified by default: a header, an API key, or something
  else?
- How is `Retry-After` computed?
- What are the details of the configuration format?

Other open points are marked `TODO:` in the sections above.
