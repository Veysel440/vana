# vana design notes

Status: draft. Section 8 describes what `vana-core` and `vana-sim` provide at
this commit: time, tickets, permits, a fixed limiter and a deterministic
simulator. The proxy, the queue, the scheduler, the adaptive limiters and
priority classes are not implemented yet; the other sections describe the
design. Points that are still open are marked `TODO:`.

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
  normal dependencies (dev-dependencies for tests are allowed).
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

- **Ticket**: a request as the core sees it: id, tenant, arrival time and an
  optional deadline. Priority class and cost are not part of a ticket yet.
  A ticket ends in exactly one way: sent upstream, rejected, expired, or
  cancelled because the client went away.
- **Permit**: the right to have one request in flight upstream. It is not an
  RAII guard. The holder hands it back with an explicit `release`, which
  consumes it, so a permit cannot be released twice. A permit that is dropped
  without release is a bug: debug builds panic, and release builds leak the
  slot silently (the limiter's `in_flight` stays too high). TODO: how the
  limiter treats samples from errors and client disconnects. TODO: how a
  dropped permit is handled in release builds once the HTTP layer cancels
  requests (a guard type that releases on drop, or a leak counter).
- **Scheduler**: priority classes, DRR between the tenants inside each class,
  and aging so that low-priority requests are not starved. TODO: the cost unit
  used by DRR (one per request, or weighted).
- **Limiter**: a trait that answers "how many requests may be in flight now"
  and grants permits. Built: Fixed. Planned: AIMD, Gradient2 and the PHP-FPM
  status limiter.
- **Clock**: a trait with one method, `now`. The core never reads a clock:
  every function that needs the time takes it as a `Nanos` argument. The proxy
  will pass real time; tests and the simulator pass virtual time.

## 8. Built so far

This section describes the code at this commit.

### time

- `Nanos(u64)` is a count of nanoseconds. It is both a point in time (counted
  from an arbitrary origin) and a duration. Arithmetic saturates: `MAX + 1` is
  `MAX` and `ZERO - 1` is `ZERO`. There are no `Add` or `Sub` operators, so
  every computation says that it saturates.
- API: `ZERO`, `MAX`, `new`, `get` (all `const`), `saturating_add`,
  `saturating_sub`. Derives `Clone`, `Copy`, `Debug`, `Default`, `PartialEq`,
  `Eq`, `PartialOrd`, `Ord`, `Hash`.
- `Clock` has one method, `fn now(&self) -> Nanos`, and implementations must
  be monotonic. `vana-core` never reads a clock. `vana-sim` provides
  `ManualClock`, which moves only when told to (`advance`, saturating).

### tickets

- `TenantId(u64)` and `TicketId(u64)` hold a raw value (`new`, `get`, both
  `const`). `TenantId` is opaque in the core: how the HTTP layer derives it is
  decided later, and it must come from a trusted source, never from a header
  a client can set freely.
- `Ticket::new(id, tenant, arrival)` has no deadline; `with_deadline` sets
  one, and a later call replaces an earlier one. Accessors: `id`, `tenant`,
  `arrival`, `deadline`. The TODOs on priority classes (section 3) and on the DRR
  cost unit (section 7) are open, so a ticket has no class or cost field.

### limiter

- `Limiter` has four methods: `limit`, `in_flight`,
  `try_acquire(now) -> Option<Permit>` and
  `release(permit, now, outcome)`. `try_acquire` is `#[must_use]`, and a
  doctest checks that ignoring its result does not compile.
- `Permit` is not `Clone` or `Copy`. Its constructor is private to
  `vana-core`, so every `Limiter` implementation lives in `vana-core`; other
  crates can use limiters but cannot add one. `acquired_at` returns the time
  of the grant. `release` consumes the permit. A permit must be released to
  the limiter that granted it; nothing checks this yet.
- Dropping a permit without release panics with "permit dropped without
  release" in debug builds, unless the thread is already panicking. In
  release builds the drop is silent and the slot stays taken.
- `Outcome` is `Success`, `Error` or `Cancelled`. The TODO on error and
  disconnect samples (section 7) is open: how a limiter counts them.
- `FixedLimiter::new(limit)` returns `LimitError::Zero` for 0 (Display:
  "limit must be at least 1"). `try_acquire` returns a permit only while
  `in_flight < limit`. `release` ignores the outcome. Releasing while
  `in_flight` is 0 panics in debug builds with "release called on a limiter
  with no permit in flight"; in release builds `in_flight` stays at 0.
- TODO: whether `Permit` carries a limiter id, so that a release to the
  wrong limiter is caught.
- TODO: I1 under a dropping limit cannot be tested with `FixedLimiter`; it is
  tested once an adaptive limiter exists.

### SplitMix64

- `SplitMix64::new(seed)` and `next_u64` implement `splitmix.c` by Sebastiano
  Vigna, <https://prng.di.unimi.it/splitmix.c> (the old `splitmix64.c`
  address redirects there). Every seed, including 0, is valid.
- `next_below(n)` is the high 64 bits of `next_u64() * n`: one draw, no
  rejection. It panics if `n` is 0. It is not exactly uniform: each value in
  `0..n` is hit by either `floor(2^64 / n)` or `ceil(2^64 / n)` of the 2^64
  possible draws. For `n` up to 2^32 two values' probabilities differ by less
  than 2^-32 of either; as `n` approaches 2^64 some values can be twice as
  likely as others.

### simulator

- `run_fixed(&FixedScenario) -> Result<Vec<TraceEvent>, ScenarioError>` runs
  synthetic traffic against a `FixedLimiter` on virtual time.
  `FixedScenario` has `seed`, `requests`, `tenants`, `limit`, `mean_gap` and
  `mean_service`.
- `TraceEvent` is `Arrive`, `Grant`, `Reject` or `Release`, each with `t`,
  `ticket`, `tenant` and `in_flight`. `to_json_line` and `to_json_lines`
  print traces.
- The same scenario gives the same trace, byte for byte (I7).

### simulation rules

1. Tickets are numbered `0..requests-1` in arrival order; the tenant is
   `ticket % tenants`.
2. For each ticket there are two draws, in this order:
   `gap = 1 + next_below(2 * mean_gap)` and
   `service = 1 + next_below(2 * mean_service)`. The multiplication
   saturates. Units are nanoseconds; there is no floating point. A rejected
   ticket still draws its service time.
3. `arrival_0 = gap_0` and `arrival_i = arrival_(i-1) + gap_i`, saturating.
4. At an arrival time `t`, first every running request whose completion time
   is at most `t` is released, in order of completion time and then ticket:
   `limiter.release(permit, completion, Success)` and a `Release` event at its
   own completion time. Then the `Arrive` event. Then `try_acquire` at the
   same `t`: if it returns a permit, a `Grant` event follows and the
   completion time is `t + service` (saturating); otherwise a `Reject` event.
5. After the last arrival, the remaining completions are processed in the
   same order.
6. `in_flight` in an event is the value after the event is applied. It does
   not change on `Arrive` and `Reject`.
7. The same `FixedScenario` always produces the same trace (I7).

#### JSON line

One event is one line with no spaces, keys in this order:

```text
{"t":<u64>,"ev":"arrive|grant|reject|release","ticket":<u64>,"tenant":<u64>,"in_flight":<u32>}
```

`to_json_lines` joins events, and every line ends with `\n`.

#### invalid scenarios

`run_fixed` checks the fields in this order and returns the error of the
first that is 0, also when `requests` is 0:

| Field | Error | Display |
|---|---|---|
| `limit` | `ZeroLimit` | limit must be at least 1 |
| `tenants` | `ZeroTenants` | tenants must be at least 1 |
| `mean_gap` | `ZeroMeanGap` | mean_gap must be at least 1 ns |
| `mean_service` | `ZeroMeanService` | mean_service must be at least 1 ns |

`requests` = 0 is valid and returns an empty trace.

## 9. Invariants

These apply when vana enforces its decisions; in shadow mode the decisions are
only recorded.

- **I1**: A new permit is granted only while in_flight < limit. If the limit
  drops below in_flight, in-flight requests are not cut; no new permit is
  granted until in_flight < limit. Tested for `FixedLimiter`; the dropping
  limit is tested once an adaptive limiter exists (see "limiter").
- **I2**: For every admitted ticket that receives a permit, the permit is
  released exactly once, including when the client disconnects; a ticket that
  ends before it receives a permit never receives one. In debug builds, a
  permit that is dropped without release panics.
- **I3**: A request whose deadline has passed is never sent upstream.
- **I4**: Within a class, every waiting tenant is served within a bounded
  number of rounds; no tenant starves.
- **I5**: Thanks to aging, a low-priority request does not wait without bound.
  TODO: the value of the bound, to be defined in Phase 1.
- **I6**: The limit always stays within `[min, max]`.
- **I7**: Given the same seed and the same input, a simulation run produces the
  same result.

## 10. Testing strategy

Correctness and performance are tested separately.

Correctness:

- Property-based tests of `vana-core`: random sequences of arrivals,
  completions, cancellations and clock steps, checked against the invariants
  above. `vana-core` has no normal dependencies; dev-dependencies for tests
  are allowed, and its property tests use `proptest`.
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

## 11. Prior art

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

## 12. Milestones

- **Phase 0**: skeleton, CI and these design notes.
- **Phase 1**: core and simulator; the invariants above are tested.
- **Phase 2**: HTTP proxy.
- **Phase 3**: adaptive limits and the PHP-FPM signal.
- **Phase 4**: the lab and comparative measurements.
- **Phase 5**: release.

## 13. Open questions

- Is the name available?
- How is a tenant identified by default: a header, an API key, or something
  else?
- How is `Retry-After` computed?
- What are the details of the configuration format?

Other open points are marked `TODO:` in the sections above.
