//! A scenario driver for a fixed limit.

use core::fmt;
use std::collections::BTreeMap;

use vana_core::{FixedLimiter, Limiter, Nanos, Outcome, Permit, TenantId, TicketId};

use crate::rng::SplitMix64;

/// Parameters of a run against a [`FixedLimiter`].
///
/// Gaps between arrivals and service times are drawn from `1..=2 * mean`, in
/// nanoseconds, with [`SplitMix64::next_below`] (nearly uniform for the
/// ranges a scenario uses), so their mean is close to the given mean.
#[derive(Clone, Debug)]
pub struct FixedScenario {
    /// Seed of the [`SplitMix64`] generator.
    pub seed: u64,
    /// Number of requests; tickets are numbered `0..requests` in arrival
    /// order.
    pub requests: u32,
    /// Number of tenants; ticket `i` belongs to tenant `i % tenants`.
    pub tenants: u64,
    /// The fixed limit.
    pub limit: u32,
    /// Mean gap between two arrivals.
    pub mean_gap: Nanos,
    /// Mean time a granted request runs before it completes.
    pub mean_service: Nanos,
}

/// One event of a run, in the order it happened.
///
/// `in_flight` is the limiter's value after the event is applied; it does
/// not change on `Arrive` and `Reject`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    /// A ticket arrived.
    Arrive {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
    },
    /// The ticket received a permit.
    Grant {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
    },
    /// The ticket was turned away because the limit was reached.
    Reject {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
    },
    /// The ticket's request completed and its permit was released.
    Release {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
    },
}

impl TraceEvent {
    /// Returns the event as one line of JSON, without a trailing newline.
    ///
    /// The line has no spaces and its keys come in this order:
    /// `{"t":..,"ev":"arrive|grant|reject|release","ticket":..,"tenant":..,"in_flight":..}`.
    pub fn to_json_line(&self) -> String {
        let (ev, t, ticket, tenant, in_flight) = match self {
            TraceEvent::Arrive {
                t,
                ticket,
                tenant,
                in_flight,
            } => ("arrive", t, ticket, tenant, in_flight),
            TraceEvent::Grant {
                t,
                ticket,
                tenant,
                in_flight,
            } => ("grant", t, ticket, tenant, in_flight),
            TraceEvent::Reject {
                t,
                ticket,
                tenant,
                in_flight,
            } => ("reject", t, ticket, tenant, in_flight),
            TraceEvent::Release {
                t,
                ticket,
                tenant,
                in_flight,
            } => ("release", t, ticket, tenant, in_flight),
        };
        format!(
            "{{\"t\":{},\"ev\":\"{}\",\"ticket\":{},\"tenant\":{},\"in_flight\":{}}}",
            t.get(),
            ev,
            ticket.get(),
            tenant.get(),
            in_flight
        )
    }
}

/// Returns the events as JSON lines, each ending with `\n`.
pub fn to_json_lines(events: &[TraceEvent]) -> String {
    let mut out = String::new();
    for event in events {
        out.push_str(&event.to_json_line());
        out.push('\n');
    }
    out
}

/// Error returned when a [`FixedScenario`] is invalid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioError {
    /// `limit` was 0.
    ZeroLimit,
    /// `tenants` was 0.
    ZeroTenants,
    /// `mean_gap` was 0.
    ZeroMeanGap,
    /// `mean_service` was 0.
    ZeroMeanService,
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            ScenarioError::ZeroLimit => "limit must be at least 1",
            ScenarioError::ZeroTenants => "tenants must be at least 1",
            ScenarioError::ZeroMeanGap => "mean_gap must be at least 1 ns",
            ScenarioError::ZeroMeanService => "mean_service must be at least 1 ns",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ScenarioError {}

/// A granted request waiting for its completion time.
struct Running {
    tenant: TenantId,
    permit: Permit,
}

/// Runs `scenario` against a [`FixedLimiter`] and returns the trace.
///
/// The fields are checked in the order `limit`, `tenants`, `mean_gap`,
/// `mean_service`, and the first one that is 0 gives the error. With
/// `requests` set to 0 the trace is empty.
///
/// For each ticket the generator is drawn twice, in this order:
/// `gap = 1 + next_below(2 * mean_gap)` and
/// `service = 1 + next_below(2 * mean_service)`, with saturating
/// multiplication. The first ticket arrives at its gap, each next one at the
/// previous arrival plus its gap. At each arrival time `t`, every request
/// whose completion time is at most `t` is released first, in order of
/// completion time and then ticket, each with a `Release` event at its own
/// completion time; then come `Arrive` and either `Grant` (completion at
/// `t + service`) or `Reject`. Requests still running after the last arrival
/// are released in the same order. All arithmetic saturates.
///
/// The same scenario always gives the same trace.
pub fn run_fixed(scenario: &FixedScenario) -> Result<Vec<TraceEvent>, ScenarioError> {
    let mut limiter = FixedLimiter::new(scenario.limit).map_err(|_| ScenarioError::ZeroLimit)?;
    if scenario.tenants == 0 {
        return Err(ScenarioError::ZeroTenants);
    }
    if scenario.mean_gap == Nanos::ZERO {
        return Err(ScenarioError::ZeroMeanGap);
    }
    if scenario.mean_service == Nanos::ZERO {
        return Err(ScenarioError::ZeroMeanService);
    }

    let gap_range = scenario.mean_gap.get().saturating_mul(2);
    let service_range = scenario.mean_service.get().saturating_mul(2);
    let mut rng = SplitMix64::new(scenario.seed);
    let mut running: BTreeMap<(Nanos, TicketId), Running> = BTreeMap::new();
    let mut events = Vec::new();
    let mut arrival = Nanos::ZERO;

    for index in 0..scenario.requests {
        let gap = Nanos::new(1u64.saturating_add(rng.next_below(gap_range)));
        let service = Nanos::new(1u64.saturating_add(rng.next_below(service_range)));
        arrival = arrival.saturating_add(gap);
        let t = arrival;

        release_until(&mut limiter, &mut running, &mut events, Some(t));

        let ticket = TicketId::new(u64::from(index));
        let tenant = TenantId::new(u64::from(index) % scenario.tenants);
        events.push(TraceEvent::Arrive {
            t,
            ticket,
            tenant,
            in_flight: limiter.in_flight(),
        });
        match limiter.try_acquire(t) {
            Some(permit) => {
                events.push(TraceEvent::Grant {
                    t,
                    ticket,
                    tenant,
                    in_flight: limiter.in_flight(),
                });
                running.insert(
                    (t.saturating_add(service), ticket),
                    Running { tenant, permit },
                );
            }
            None => events.push(TraceEvent::Reject {
                t,
                ticket,
                tenant,
                in_flight: limiter.in_flight(),
            }),
        }
    }

    release_until(&mut limiter, &mut running, &mut events, None);
    Ok(events)
}

/// Releases, in key order, every running request whose completion time is at
/// most `until` (all of them when `until` is `None`).
fn release_until(
    limiter: &mut FixedLimiter,
    running: &mut BTreeMap<(Nanos, TicketId), Running>,
    events: &mut Vec<TraceEvent>,
    until: Option<Nanos>,
) {
    while let Some(entry) = running.first_entry() {
        let (done, ticket) = *entry.key();
        if until.is_some_and(|t| done > t) {
            break;
        }
        let Running { tenant, permit } = entry.remove();
        limiter.release(permit, done, Outcome::Success);
        events.push(TraceEvent::Release {
            t: done,
            ticket,
            tenant,
            in_flight: limiter.in_flight(),
        });
    }
}
