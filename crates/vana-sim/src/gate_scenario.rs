//! A scenario driver for a [`Gate`] in front of a fixed limit.

use std::collections::BTreeMap;

use vana_core::{
    Admission, Dequeued, FixedLimiter, Gate, Limiter, Nanos, Outcome, Permit, RejectReason,
    TenantId, Ticket, TicketId,
};

use crate::rng::SplitMix64;
use crate::scenario::ScenarioError;

/// Parameters of a run against a [`Gate`] in front of a [`FixedLimiter`].
///
/// Tickets, tenants, gaps and service times are drawn exactly as in
/// [`run_fixed`](crate::run_fixed).
#[derive(Clone, Debug)]
pub struct GateScenario {
    /// Seed of the [`SplitMix64`] generator.
    pub seed: u64,
    /// Number of requests; tickets are numbered `0..requests` in arrival
    /// order.
    pub requests: u32,
    /// Number of tenants; ticket `i` belongs to tenant `i % tenants`.
    pub tenants: u64,
    /// The fixed limit.
    pub limit: u32,
    /// Room in the gate's queue; 0 means no queue.
    pub queue_capacity: u32,
    /// Each ticket's deadline is its arrival plus `timeout`; `None` means
    /// tickets have no deadline.
    pub timeout: Option<Nanos>,
    /// Mean gap between two arrivals.
    pub mean_gap: Nanos,
    /// Mean time a granted request runs before it completes.
    pub mean_service: Nanos,
}

/// One event of a gate run, in the order it happened.
///
/// `in_flight` and `queued` are the values after the event is applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateEvent {
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
        /// Tickets in the queue after the event.
        queued: u32,
    },
    /// The ticket received a permit, on arrival or from the queue.
    Grant {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
        /// Tickets in the queue after the event.
        queued: u32,
    },
    /// The ticket entered the queue.
    Queue {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
        /// Tickets in the queue after the event.
        queued: u32,
    },
    /// The ticket was turned away on arrival.
    Reject {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
        /// Tickets in the queue after the event.
        queued: u32,
        /// Why the ticket was turned away.
        reason: RejectReason,
    },
    /// The ticket left the queue because its deadline passed.
    Expire {
        /// Time of the event.
        t: Nanos,
        /// The ticket.
        ticket: TicketId,
        /// The ticket's tenant.
        tenant: TenantId,
        /// Permits held after the event.
        in_flight: u32,
        /// Tickets in the queue after the event.
        queued: u32,
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
        /// Tickets in the queue after the event.
        queued: u32,
    },
}

impl GateEvent {
    /// Returns the event as one line of JSON, without a trailing newline.
    ///
    /// The line has no spaces and its keys come in this order:
    /// `{"t":..,"ev":"arrive|grant|queue|reject|expire|release","ticket":..,"tenant":..,"in_flight":..,"queued":..}`;
    /// a reject line ends with `,"reason":"queue_full|expired"` before the
    /// closing brace.
    pub fn to_json_line(&self) -> String {
        let (ev, t, ticket, tenant, in_flight, queued, reason) = match self {
            GateEvent::Arrive {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
            } => ("arrive", t, ticket, tenant, in_flight, queued, None),
            GateEvent::Grant {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
            } => ("grant", t, ticket, tenant, in_flight, queued, None),
            GateEvent::Queue {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
            } => ("queue", t, ticket, tenant, in_flight, queued, None),
            GateEvent::Reject {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
                reason,
            } => ("reject", t, ticket, tenant, in_flight, queued, Some(reason)),
            GateEvent::Expire {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
            } => ("expire", t, ticket, tenant, in_flight, queued, None),
            GateEvent::Release {
                t,
                ticket,
                tenant,
                in_flight,
                queued,
            } => ("release", t, ticket, tenant, in_flight, queued, None),
        };
        let mut line = format!(
            "{{\"t\":{},\"ev\":\"{}\",\"ticket\":{},\"tenant\":{},\"in_flight\":{},\"queued\":{}",
            t.get(),
            ev,
            ticket.get(),
            tenant.get(),
            in_flight,
            queued
        );
        if let Some(reason) = reason {
            let name = match reason {
                RejectReason::QueueFull => "queue_full",
                RejectReason::Expired => "expired",
            };
            line.push_str(",\"reason\":\"");
            line.push_str(name);
            line.push('"');
        }
        line.push('}');
        line
    }
}

/// Returns the events as JSON lines, each ending with `\n`.
pub fn gate_json_lines(events: &[GateEvent]) -> String {
    let mut out = String::new();
    for event in events {
        out.push_str(&event.to_json_line());
        out.push('\n');
    }
    out
}

/// A granted request waiting for its completion time.
struct Running {
    tenant: TenantId,
    permit: Permit,
}

/// The next ticket to arrive, with its draws.
struct Pending {
    ticket: TicketId,
    tenant: TenantId,
    arrival: Nanos,
    service: Nanos,
}

/// Draws the next ticket of a run: its gap, then its service time.
struct Arrivals {
    rng: SplitMix64,
    gap_range: u64,
    service_range: u64,
    tenants: u64,
    requests: u32,
    drawn: u32,
    last_arrival: Nanos,
}

impl Arrivals {
    fn next(&mut self) -> Option<Pending> {
        if self.drawn == self.requests {
            return None;
        }
        let index = self.drawn;
        self.drawn += 1;
        let gap = Nanos::new(1u64.saturating_add(self.rng.next_below(self.gap_range)));
        let service = Nanos::new(1u64.saturating_add(self.rng.next_below(self.service_range)));
        self.last_arrival = self.last_arrival.saturating_add(gap);
        Some(Pending {
            ticket: TicketId::new(u64::from(index)),
            tenant: TenantId::new(u64::from(index) % self.tenants),
            arrival: self.last_arrival,
            service,
        })
    }
}

/// Runs `scenario` against a [`Gate`] in front of a [`FixedLimiter`] and
/// returns the trace.
///
/// The fields are checked as in [`run_fixed`](crate::run_fixed): `limit`,
/// `tenants`, `mean_gap`, `mean_service`, and the first one that is 0 gives
/// the error. `queue_capacity` 0 and a `timeout` of 0 are valid.
///
/// Draws, ticket numbers and tenants are the same as in `run_fixed`. A
/// ticket's deadline is its arrival plus `timeout` (saturating). Time moves
/// to the earliest of: the next completion, the earliest deadline in the
/// queue, the next arrival. At that time `t`, in this order:
///
/// 1. every running request whose completion time is at most `t` is
///    released, in order of completion time and then ticket: a `Release`
///    event, then one `Expire` or `Grant` event for each ticket the gate
///    moved out of the queue, in order. A ticket granted from the queue
///    completes at `t` plus its own service time;
/// 2. every expired ticket still in the queue leaves it, in queue order,
///    with an `Expire` event;
/// 3. if a ticket arrives at `t`: `Arrive`, then `Grant`, `Queue` or
///    `Reject`. Several arrivals at the same time (possible only when time
///    saturates) are handled one per round.
///
/// The run ends when every ticket has ended. The same scenario always gives
/// the same trace.
pub fn run_gate(scenario: &GateScenario) -> Result<Vec<GateEvent>, ScenarioError> {
    let limiter = FixedLimiter::new(scenario.limit).map_err(|_| ScenarioError::ZeroLimit)?;
    if scenario.tenants == 0 {
        return Err(ScenarioError::ZeroTenants);
    }
    if scenario.mean_gap == Nanos::ZERO {
        return Err(ScenarioError::ZeroMeanGap);
    }
    if scenario.mean_service == Nanos::ZERO {
        return Err(ScenarioError::ZeroMeanService);
    }

    let mut gate = Gate::new(limiter, scenario.queue_capacity);
    let mut arrivals = Arrivals {
        rng: SplitMix64::new(scenario.seed),
        gap_range: scenario.mean_gap.get().saturating_mul(2),
        service_range: scenario.mean_service.get().saturating_mul(2),
        tenants: scenario.tenants,
        requests: scenario.requests,
        drawn: 0,
        last_arrival: Nanos::ZERO,
    };
    let mut next = arrivals.next();
    let mut running: BTreeMap<(Nanos, TicketId), Running> = BTreeMap::new();
    // Service times of the tickets waiting in the queue.
    let mut waiting: BTreeMap<TicketId, Nanos> = BTreeMap::new();
    let mut events = Vec::new();

    loop {
        let candidates = [
            running.first_key_value().map(|(&(done, _), _)| done),
            gate.next_deadline(),
            next.as_ref().map(|pending| pending.arrival),
        ];
        let Some(t) = candidates.into_iter().flatten().min() else {
            break;
        };

        // 1. Completions.
        while let Some(entry) = running.first_entry() {
            let (done, ticket) = *entry.key();
            if done > t {
                break;
            }
            let Running { tenant, permit } = entry.remove();
            let mut in_flight = gate.limiter().in_flight().saturating_sub(1);
            let mut queued = gate.queued();
            let moved = gate.release(permit, done, Outcome::Success);
            events.push(GateEvent::Release {
                t: done,
                ticket,
                tenant,
                in_flight,
                queued,
            });
            for item in moved {
                queued = queued.saturating_sub(1);
                match item {
                    Dequeued::Expired(expired) => {
                        waiting.remove(&expired.id());
                        events.push(GateEvent::Expire {
                            t: done,
                            ticket: expired.id(),
                            tenant: expired.tenant(),
                            in_flight,
                            queued,
                        });
                    }
                    Dequeued::Granted(granted, permit) => {
                        in_flight += 1;
                        let service = waiting.remove(&granted.id()).unwrap_or(Nanos::ZERO);
                        events.push(GateEvent::Grant {
                            t: done,
                            ticket: granted.id(),
                            tenant: granted.tenant(),
                            in_flight,
                            queued,
                        });
                        running.insert(
                            (done.saturating_add(service), granted.id()),
                            Running {
                                tenant: granted.tenant(),
                                permit,
                            },
                        );
                    }
                }
            }
        }

        // 2. Deadlines in the queue.
        let mut queued = gate.queued();
        for expired in gate.expire(t) {
            queued = queued.saturating_sub(1);
            waiting.remove(&expired.id());
            events.push(GateEvent::Expire {
                t,
                ticket: expired.id(),
                tenant: expired.tenant(),
                in_flight: gate.limiter().in_flight(),
                queued,
            });
        }

        // 3. One arrival.
        if let Some(pending) = next.take_if(|pending| pending.arrival == t) {
            let Pending {
                ticket,
                tenant,
                arrival,
                service,
            } = pending;
            events.push(GateEvent::Arrive {
                t,
                ticket,
                tenant,
                in_flight: gate.limiter().in_flight(),
                queued: gate.queued(),
            });
            let mut request = Ticket::new(ticket, tenant, arrival);
            if let Some(timeout) = scenario.timeout {
                request = request.with_deadline(arrival.saturating_add(timeout));
            }
            match gate.submit(request, t) {
                Admission::Granted(_, permit) => {
                    events.push(GateEvent::Grant {
                        t,
                        ticket,
                        tenant,
                        in_flight: gate.limiter().in_flight(),
                        queued: gate.queued(),
                    });
                    running.insert(
                        (t.saturating_add(service), ticket),
                        Running { tenant, permit },
                    );
                }
                Admission::Queued => {
                    waiting.insert(ticket, service);
                    events.push(GateEvent::Queue {
                        t,
                        ticket,
                        tenant,
                        in_flight: gate.limiter().in_flight(),
                        queued: gate.queued(),
                    });
                }
                Admission::Rejected(_, reason) => events.push(GateEvent::Reject {
                    t,
                    ticket,
                    tenant,
                    in_flight: gate.limiter().in_flight(),
                    queued: gate.queued(),
                    reason,
                }),
            }
            next = arrivals.next();
        }
    }

    Ok(events)
}
