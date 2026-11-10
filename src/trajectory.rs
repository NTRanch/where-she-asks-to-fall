//! The canonical cross-platform fixture: one scripted
//! run, executed identically on every platform, and the
//! FNV-1a digest of its per-step record.
//!
//! The digest discipline is the coin pilot's construction
//! (itself the estate's `state_hash` discipline),
//! re-implemented here so this crate gains no dependency
//! on the app crate: FNV-1a 64-bit (offset basis
//! `0xcbf29ce484222325`, prime `0x100000001b3`) over the
//! little-endian byte stream of each record's words, in
//! order. The record here is the state snapshot after
//! each fixture step: day, block, layer, the anchor mask,
//! the six meters, every lesson's counts, the beat and
//! completion flags, the unlock date, and the mantra
//! history. Equal digests on two platforms mean the two
//! runs walked the same states at the same steps, bit
//! for bit — the S7 cross-platform determinism gate
//! (PIPELINE.md, D-212).

use crate::date::CalendarDate;
use crate::protocol::{FallError, LESSON_COUNT, Protocol, design_protocol};
use crate::state::{MantraOutcome, State};

/// FNV-1a offset basis (64-bit).
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime (64-bit).
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// An incremental FNV-1a hasher over bytes, fed with
/// little-endian u64 words or raw byte strings.
struct TrajectoryHasher {
    state: u64,
}

impl TrajectoryHasher {
    fn new() -> TrajectoryHasher {
        TrajectoryHasher {
            state: FNV_OFFSET_BASIS,
        }
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.state ^= u64::from(*byte);
            self.state = self.state.wrapping_mul(FNV_PRIME);
        }
    }

    fn write_u64(&mut self, value: u64) {
        self.write_bytes(&value.to_le_bytes());
    }

    fn finish(&self) -> u64 {
        self.state
    }
}

/// The S7 state hash: FNV-1a over the state's canonical
/// serialized save bytes. Same state, same bytes, same
/// hash — on every platform.
pub fn hash_state(state: &State) -> u64 {
    let mut hasher = TrajectoryHasher::new();
    // A validated state always serializes; the empty
    // stream (which hashes to the offset basis) is the
    // documented fallback for the unreachable failure.
    if let Ok(text) = state.to_ron() {
        hasher.write_bytes(text.as_bytes());
    }
    hasher.finish()
}

fn outcome_word(outcome: MantraOutcome) -> u64 {
    match outcome {
        MantraOutcome::Pass => 0,
        MantraOutcome::Fail => 1,
        MantraOutcome::Assisted => 2,
    }
}

/// Snapshot the state into the hasher, in a fixed word
/// order.
fn write_snapshot(hasher: &mut TrajectoryHasher, state: &State) {
    hasher.write_u64(u64::from(state.day()));
    hasher.write_u64(u64::from(state.block()));
    hasher.write_u64(u64::from(state.layer()));
    hasher.write_u64(u64::from(state.anchors()));
    let meters = state.meters();
    for value in [
        meters.depth,
        meters.craving,
        meters.yes,
        meters.pairing,
        meters.ritual,
        meters.spiral,
    ] {
        hasher.write_u64(u64::from(value));
    }
    for lesson in 1..=u8::try_from(LESSON_COUNT).unwrap_or(0) {
        if let Some(counts) = state.counts(lesson) {
            hasher.write_u64(u64::from(counts.deep));
            hasher.write_u64(u64::from(counts.practice));
        }
    }
    hasher.write_u64(u64::from(state.mantra_passed_unassisted()));
    hasher.write_u64(u64::from(state.anchor_done()));
    hasher.write_u64(u64::from(state.activity_done()));
    hasher.write_u64(u64::from(state.ledger_pending()));
    hasher.write_u64(u64::from(state.is_finished()));
    hasher.write_u64(u64::from(state.is_looped()));
    match state.unlock_date() {
        Some(date) => {
            hasher.write_u64(u64::try_from(date.year()).unwrap_or(0));
            hasher.write_u64(u64::from(date.month()));
            hasher.write_u64(u64::from(date.day()));
        }
        None => {
            hasher.write_u64(0);
            hasher.write_u64(0);
            hasher.write_u64(0);
        }
    }
    hasher.write_u64(u64::try_from(state.mantra_history().len()).unwrap_or(0));
    for record in state.mantra_history() {
        hasher.write_bytes(record.line.as_bytes());
        hasher.write_u64(outcome_word(record.outcome));
    }
}

/// The result of one fixture run.
#[derive(Clone, Debug, PartialEq)]
pub struct FixtureRun {
    /// FNV-1a digest of the run's per-step record.
    pub trajectory_hash: u64,
    /// The S7 state hash of the final state.
    pub final_state_hash: u64,
    /// Fixture steps executed (one snapshot per step).
    pub steps_executed: usize,
    /// Protocol days completed by the fixture (10).
    pub days_completed: u16,
    /// The anchor mask at the end (lesson 1's cue).
    pub anchors_installed: u16,
    /// The Depth meter at the end.
    pub depth_meter: u32,
}

/// Run the canonical fixture over `protocol`: the first
/// block of the protocol compressed into ten injected
/// calendar days — Permission, then each day's four
/// beats in order. The same code runs in `fall-smoke`
/// on Linux and in any future Android shell; the
/// printed hashes are the S7 comparison values.
pub fn run_fixture(protocol: &Protocol) -> Result<FixtureRun, FallError> {
    let mut state = State::new();
    let mut hasher = TrajectoryHasher::new();
    let mut steps = 0usize;
    let mut date = CalendarDate::new(2027, 3, 1)?;

    macro_rules! step {
        () => {{
            write_snapshot(&mut hasher, &state);
            steps += 1;
        }};
    }

    state.grant_permission(date)?;
    step!();
    for _day in 1..=10u16 {
        let cue = if state.anchors() == 0 { None } else { Some(1) };
        state.record_anchor_check(cue, date)?;
        step!();
        state.record_activity(protocol)?;
        step!();
        let day = state.day();
        let row = protocol.day_row(day)?;
        state.complete_session(protocol, row.lesson, row.session, date)?;
        step!();
        state.record_ledger()?;
        step!();
        if let Some(next) = date.next_day() {
            date = next;
        }
    }

    Ok(FixtureRun {
        trajectory_hash: hasher.finish(),
        final_state_hash: hash_state(&state),
        steps_executed: steps,
        days_completed: 10,
        anchors_installed: state.anchors(),
        depth_meter: state.meter(crate::MeterKind::Depth),
    })
}

/// The fixture over the design protocol tables.
pub fn run_design_fixture() -> Result<FixtureRun, FallError> {
    run_fixture(&design_protocol()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stream_hashes_to_the_offset_basis() {
        assert_eq!(TrajectoryHasher::new().finish(), FNV_OFFSET_BASIS);
    }

    #[test]
    fn fixture_is_deterministic_across_runs() {
        let a = run_design_fixture().expect("first run");
        let b = run_design_fixture().expect("second run");
        assert_eq!(a, b);
        assert_eq!(a.days_completed, 10);
        assert_eq!(a.anchors_installed, 0b1);
        assert_eq!(a.steps_executed, 41);
    }
}
