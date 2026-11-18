//! Property tests: random operation sequences on `Gate` against a plain
//! reference model written from the rules in the design notes.

use proptest::prelude::*;
use vana_core::{
    Admission, Dequeued, FixedLimiter, Gate, Limiter, Nanos, Outcome, Permit, RejectReason,
    TenantId, Ticket, TicketId,
};

/// One random step. Time never goes back: each step first advances the clock
/// by `advance` nanoseconds.
#[derive(Clone, Debug)]
enum Op {
    /// Submit a ticket whose deadline is `offset` after now, if any.
    Submit { advance: u64, offset: Option<u64> },
    /// Release the running permit at this position (modulo how many run).
    Release { advance: u64, pick: usize },
    /// Run `expire` at the current time.
    Expire { advance: u64 },
}

fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        5 => (0u64..4, prop::option::of(0u64..10))
            .prop_map(|(advance, offset)| Op::Submit { advance, offset }),
        3 => (0u64..4, any::<usize>()).prop_map(|(advance, pick)| Op::Release { advance, pick }),
        2 => (0u64..4).prop_map(|advance| Op::Expire { advance }),
    ]
}

/// The model: a queue of (ticket, deadline) and a count of running requests.
struct Model {
    limit: u32,
    capacity: u32,
    running: u32,
    queue: Vec<(u64, Option<u64>)>,
}

fn expired(deadline: Option<u64>, now: u64) -> bool {
    deadline.is_some_and(|d| d <= now)
}

/// Plays `ops` and returns the first violation. Every permit is released
/// before returning.
fn play(limit: u32, capacity: u32, ops: &[Op]) -> Result<(), String> {
    let mut gate = Gate::new(
        FixedLimiter::new(limit).map_err(|e| e.to_string())?,
        capacity,
    );
    let mut model = Model {
        limit,
        capacity,
        running: 0,
        queue: Vec::new(),
    };
    // Running permits with the id of their ticket.
    let mut held: Vec<(u64, Permit)> = Vec::new();
    let mut now: u64 = 0;
    let mut next_id: u64 = 0;
    // Ids of queued tickets in grant order, to check first in, first out.
    let mut last_granted_from_queue: Option<u64> = None;
    let mut problem: Option<String> = None;

    for (step, op) in ops.iter().enumerate() {
        if problem.is_some() {
            break;
        }
        match op {
            Op::Submit { advance, offset } => {
                now += advance;
                let id = next_id;
                next_id += 1;
                let deadline = offset.map(|o| now + o);
                let mut t = Ticket::new(TicketId::new(id), TenantId::new(id), Nanos::new(now));
                if let Some(d) = deadline {
                    t = t.with_deadline(Nanos::new(d));
                }
                // The model's answer, from the rules.
                let want = if expired(deadline, now) {
                    "expired"
                } else if model.queue.is_empty() && model.running < model.limit {
                    "granted"
                } else if (model.queue.len() as u32) < model.capacity {
                    "queued"
                } else {
                    "full"
                };
                match gate.submit(t, Nanos::new(now)) {
                    Admission::Granted(t, p) => {
                        if want != "granted" {
                            problem = Some(format!("step {step}: granted, model says {want}"));
                        }
                        if t.id().get() != id {
                            problem = Some(format!("step {step}: wrong ticket granted"));
                        }
                        if expired(deadline, now) {
                            problem = Some(format!("step {step}: I3 an expired ticket granted"));
                        }
                        model.running += 1;
                        held.push((id, p));
                    }
                    Admission::Queued => {
                        if want != "queued" {
                            problem = Some(format!("step {step}: queued, model says {want}"));
                        }
                        model.queue.push((id, deadline));
                    }
                    Admission::Rejected(t, reason) => {
                        let got = match reason {
                            RejectReason::Expired => "expired",
                            RejectReason::QueueFull => "full",
                        };
                        if got != want || t.id().get() != id {
                            problem = Some(format!("step {step}: rejected {got}, model {want}"));
                        }
                    }
                }
            }
            Op::Release { advance, pick } => {
                now += advance;
                if held.is_empty() {
                    continue;
                }
                let (_, permit) = held.swap_remove(pick % held.len());
                model.running -= 1;
                // The model drains the head until the limiter is full.
                let mut want: Vec<(&str, u64)> = Vec::new();
                while !model.queue.is_empty() {
                    let (id, deadline) = model.queue[0];
                    if expired(deadline, now) {
                        model.queue.remove(0);
                        want.push(("expired", id));
                    } else if model.running < model.limit {
                        model.queue.remove(0);
                        model.running += 1;
                        want.push(("granted", id));
                    } else {
                        break;
                    }
                }
                let got = gate.release(permit, Nanos::new(now), Outcome::Success);
                let mut seen: Vec<(&str, u64)> = Vec::new();
                for d in got {
                    match d {
                        Dequeued::Granted(t, p) => {
                            let id = t.id().get();
                            if expired(t.deadline().map(|d| d.get()), now) {
                                problem =
                                    Some(format!("step {step}: I3 expired ticket {id} granted"));
                            }
                            if p.acquired_at() != Nanos::new(now) {
                                problem = Some(format!("step {step}: permit time is not now"));
                            }
                            if last_granted_from_queue.is_some_and(|last| last > id) {
                                problem = Some(format!("step {step}: FIFO broken at ticket {id}"));
                            }
                            last_granted_from_queue = Some(id);
                            seen.push(("granted", id));
                            held.push((id, p));
                        }
                        Dequeued::Expired(t) => seen.push(("expired", t.id().get())),
                    }
                }
                if seen != want {
                    problem = Some(format!(
                        "step {step}: release gave {seen:?}, model {want:?}"
                    ));
                }
            }
            Op::Expire { advance } => {
                now += advance;
                let want: Vec<u64> = model
                    .queue
                    .iter()
                    .filter(|(_, d)| expired(*d, now))
                    .map(|(id, _)| *id)
                    .collect();
                model.queue.retain(|(_, d)| !expired(*d, now));
                let got: Vec<u64> = gate
                    .expire(Nanos::new(now))
                    .into_iter()
                    .map(|t| t.id().get())
                    .collect();
                if got != want {
                    problem = Some(format!("step {step}: expire gave {got:?}, model {want:?}"));
                }
            }
        }
        if problem.is_some() {
            break;
        }
        // The invariants, after every public operation.
        let in_flight = gate.limiter().in_flight();
        if in_flight > limit {
            problem = Some(format!(
                "step {step}: I1 in_flight {in_flight} > limit {limit}"
            ));
        } else if in_flight != model.running || in_flight as usize != held.len() {
            problem = Some(format!(
                "step {step}: in_flight {in_flight} != model {}",
                model.running
            ));
        } else if gate.queued() as usize != model.queue.len() {
            problem = Some(format!(
                "step {step}: queued {} != model {}",
                gate.queued(),
                model.queue.len()
            ));
        } else if gate.queued() > capacity {
            problem = Some(format!(
                "step {step}: queued {} > capacity {capacity}",
                gate.queued()
            ));
        } else if gate.queued() > 0 && in_flight != limit {
            problem = Some(format!(
                "step {step}: I8 queued {} with in_flight {in_flight}",
                gate.queued()
            ));
        } else {
            let want = model.queue.iter().filter_map(|(_, d)| *d).min();
            if gate.next_deadline().map(|d| d.get()) != want {
                problem = Some(format!(
                    "step {step}: next_deadline {:?}, model {want:?}",
                    gate.next_deadline()
                ));
            }
        }
    }

    // I2: every permit is released exactly once; drain what is left. Tickets
    // granted from the queue during the drain are released too.
    while let Some((_, permit)) = held.pop() {
        now += 1;
        for d in gate.release(permit, Nanos::new(now), Outcome::Success) {
            if let Dequeued::Granted(t, p) = d {
                held.push((t.id().get(), p));
            }
        }
    }
    if problem.is_none() && (gate.limiter().in_flight() != 0 || gate.queued() != 0) {
        problem = Some(format!(
            "after draining: in_flight {}, queued {}",
            gate.limiter().in_flight(),
            gate.queued()
        ));
    }
    match problem {
        Some(message) => Err(message),
        None => Ok(()),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    // I1, I3, I8, FIFO and the model's answers hold over long random sequences.
    #[test]
    fn i1_i3_i8_hold_and_gate_matches_the_rules_over_random_sequences(
        limit in 1u32..5,
        capacity in 0u32..6,
        ops in prop::collection::vec(op(), 0..120),
    ) {
        prop_assert_eq!(play(limit, capacity, &ops), Ok(()));
    }
}
