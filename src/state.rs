//! The protocol state: the design doc's section-4
//! vocabulary, and nothing outside it.
//!
//! The state is the whole save file. Every transition
//! validates fully before it mutates — a refused
//! transition leaves the state byte-identical, which the
//! crate's adversarial tests assert. Dates enter as
//! values; the core reads no clock, no filesystem, no
//! environment.
//!
//! The day loop (design section 3) is enforced as an
//! order: anchor -> activity -> session -> ledger. The
//! session beat closes the day: completing the day's
//! scheduled session advances the counter and sets the
//! next day's unlock date to the following calendar
//! date (calendar pacing, D-1140 — one new protocol day
//! per real day). The ledger beat then seals the
//! completed day before the next day's beats may start.
//! Practice and Maintenance replays of installed lessons
//! are always available and never touch the day, the
//! clock, or the beats.

use serde::{Deserialize, Serialize};

use crate::date::CalendarDate;
use crate::protocol::{
    FallError, LESSON_COUNT, MANTRA_TEXT_LEN_MAX, METER_MAX, MeterKind, Protocol, SessionKind,
    layer_for_day,
};

/// The save format version this crate reads and writes.
/// A save from a future version is a typed refusal.
pub const SAVE_FORMAT_VERSION: u32 = 1;
/// Mantra history kept in the save: the last 8 attempts
/// (design section 4).
pub const MANTRA_HISTORY_MAX: usize = 8;
/// Failed attempts on one mantra line after which the
/// keeper speaks the line and the attempt records as
/// assisted (design section 5: three attempts).
pub const MANTRA_ATTEMPTS_FOR_ASSIST: usize = 3;
/// Maintenance editions are native from this lesson
/// (design section 3).
pub const MAINTENANCE_FIRST_LESSON: u8 = 5;
/// A bound on a mantra line id stored in the save.
pub const MANTRA_ID_LEN_MAX: usize = 64;

/// What one mantra attempt came to.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum MantraOutcome {
    /// Heard text matched the line, unassisted.
    Pass,
    /// The keeper spoke the line after three failures;
    /// the day proceeds (design section 5).
    Assisted,
    /// Heard text did not match.
    Fail,
}

/// One recorded mantra attempt (only the outcome and the
/// line persist — the user's voice audio never does,
/// design section 8).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MantraRecord {
    /// The mantra line's id.
    pub line: String,
    /// How the attempt came out.
    pub outcome: MantraOutcome,
}

/// Per-lesson completion counts. Maintenance completions
/// count as practice: the section-4 vocabulary has two
/// counters, and a maintenance edition is a practice
/// variant.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LessonCounts {
    /// Deep (Full edition) completions.
    pub deep: u32,
    /// Practice (Lite/Maintenance) completions.
    pub practice: u32,
}

/// The six layer meters, each 0..=1000.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Meters {
    /// Layer 1 (Lux).
    pub depth: u32,
    /// Layer 2 (Yui).
    pub craving: u32,
    /// Layer 3 (Piper).
    pub yes: u32,
    /// Layer 4 (Sloane).
    pub pairing: u32,
    /// Layer 5 (Alma).
    pub ritual: u32,
    /// Layer 6 (Prima).
    pub spiral: u32,
}

impl Meters {
    /// One meter's value.
    pub fn get(&self, kind: MeterKind) -> u32 {
        match kind {
            MeterKind::Depth => self.depth,
            MeterKind::Craving => self.craving,
            MeterKind::Yes => self.yes,
            MeterKind::Pairing => self.pairing,
            MeterKind::Ritual => self.ritual,
            MeterKind::Spiral => self.spiral,
        }
    }

    /// Add `gain` to one meter, saturating at the cap —
    /// a meter can never overflow or wrap.
    fn add(&mut self, kind: MeterKind, gain: u32) {
        let slot = match kind {
            MeterKind::Depth => &mut self.depth,
            MeterKind::Craving => &mut self.craving,
            MeterKind::Yes => &mut self.yes,
            MeterKind::Pairing => &mut self.pairing,
            MeterKind::Ritual => &mut self.ritual,
            MeterKind::Spiral => &mut self.spiral,
        };
        *slot = slot.saturating_add(gain).min(METER_MAX);
    }
}

/// The spiral visual's style (design section 3: the
/// spiral and pendulum are customizable).
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SpiralStyle {
    /// The ladder's early amber grade.
    #[default]
    Amber,
    /// The ladder's late violet grade.
    Violet,
    /// A plain monochrome spiral.
    Mono,
}

/// Where the always-on surface control sits (design
/// section 2: never hidden, one tap away).
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SurfacePosition {
    TopLeft,
    TopRight,
    BottomLeft,
    /// The default corner.
    #[default]
    BottomRight,
}

/// Player settings carried in the save (design section
/// 4).
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    /// Spiral visual style.
    pub spiral_style: SpiralStyle,
    /// Pendulum shown alongside the spiral.
    pub pendulum: bool,
    /// Session volume, 0..=100 percent.
    pub volume_percent: u8,
    /// Surface-control corner.
    pub surface_position: SurfacePosition,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            spiral_style: SpiralStyle::Amber,
            pendulum: true,
            volume_percent: 80,
            surface_position: SurfacePosition::BottomRight,
        }
    }
}

impl Settings {
    /// The settings bounds, checked on load and on
    /// update.
    fn validate(&self) -> Result<(), FallError> {
        if self.volume_percent > 100 {
            return Err(FallError::InvalidSettings {
                reason: "volume_percent above 100",
            });
        }
        Ok(())
    }
}

/// What a session completion did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEffect {
    /// The current day's scheduled session: the day
    /// closed and the counter advanced (or finished).
    Scheduled,
    /// A free replay of an installed lesson: counts and
    /// meter moved; day, clock, and beats did not.
    Replay,
}

/// The protocol state — the whole save file (design
/// section 4). `block` and `layer` are stored beside
/// `day` for save-format stability and verified against
/// the derivation at load.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct State {
    format_version: u32,
    day: u16,
    block: u8,
    layer: u8,
    anchors: u16,
    meters: Meters,
    sessions: [LessonCounts; LESSON_COUNT],
    mantra_history: Vec<MantraRecord>,
    mantra_passed_unassisted: bool,
    unlock_date: Option<CalendarDate>,
    anchor_done: bool,
    activity_done: bool,
    ledger_pending: bool,
    finished: bool,
    looped: bool,
    settings: Settings,
}

impl Default for State {
    fn default() -> State {
        State::new()
    }
}

/// The layer a lesson belongs to (lessons 1-2 layer 1,
/// ..., lesson 10 layer 6), via its block's first day.
fn layer_of_lesson(lesson: u8) -> u8 {
    let first_day = (u16::from(lesson) - 1) * 10 + 1;
    layer_for_day(first_day).expect("a valid lesson's first day has a layer")
}

/// Normalize heard mantra text for comparison: trim,
/// case-fold, collapse whitespace runs to one space,
/// and drop one trailing period (speech recognizers are
/// unreliable about final punctuation). The same
/// normalization is applied to the expected line.
fn normalize_mantra(text: &str) -> String {
    let folded = text.trim().to_lowercase();
    let collapsed = folded.split_whitespace().collect::<Vec<_>>().join(" ");
    match collapsed.strip_suffix('.') {
        Some(stripped) => stripped.trim_end().to_string(),
        None => collapsed,
    }
}

impl State {
    /// A fresh state: day 0, before Permission.
    pub fn new() -> State {
        State {
            format_version: SAVE_FORMAT_VERSION,
            day: 0,
            block: 0,
            layer: 0,
            anchors: 0,
            meters: Meters::default(),
            sessions: [LessonCounts::default(); LESSON_COUNT],
            mantra_history: Vec::new(),
            mantra_passed_unassisted: false,
            unlock_date: None,
            anchor_done: false,
            activity_done: false,
            ledger_pending: false,
            finished: false,
            looped: false,
            settings: Settings::default(),
        }
    }

    /// The current protocol day (0 = before Permission).
    pub fn day(&self) -> u16 {
        self.day
    }

    /// The stored block (derived from the day).
    pub fn block(&self) -> u8 {
        self.block
    }

    /// The stored layer (0 before Permission).
    pub fn layer(&self) -> u8 {
        self.layer
    }

    /// The installed-cue bitmask: bit (lesson - 1) set
    /// means that lesson's anchor is installed.
    pub fn anchors(&self) -> u16 {
        self.anchors
    }

    /// Whether lesson `lesson`'s anchor is installed.
    pub fn has_anchor(&self, lesson: u8) -> bool {
        (1..=10).contains(&lesson) && self.anchors & (1u16 << (lesson - 1)) != 0
    }

    /// All six meters.
    pub fn meters(&self) -> Meters {
        self.meters
    }

    /// One meter's value.
    pub fn meter(&self, kind: MeterKind) -> u32 {
        self.meters.get(kind)
    }

    /// One lesson's completion counts, if the lesson
    /// exists.
    pub fn counts(&self, lesson: u8) -> Option<LessonCounts> {
        if !(1..=10).contains(&lesson) {
            return None;
        }
        Some(self.sessions[usize::from(lesson - 1)])
    }

    /// The bounded mantra history, oldest first.
    pub fn mantra_history(&self) -> &[MantraRecord] {
        &self.mantra_history
    }

    /// Whether an unassisted mantra pass has ever been
    /// recorded (Piper's exit requires one; the flag is
    /// cumulative so the bounded history window cannot
    /// un-satisfy the exit).
    pub fn mantra_passed_unassisted(&self) -> bool {
        self.mantra_passed_unassisted
    }

    /// The local date on which the current day unlocked.
    pub fn unlock_date(&self) -> Option<CalendarDate> {
        self.unlock_date
    }

    /// Whether today's anchor beat is done.
    pub fn anchor_done(&self) -> bool {
        self.anchor_done
    }

    /// Whether today's activity beat is done.
    pub fn activity_done(&self) -> bool {
        self.activity_done
    }

    /// Whether a completed day is waiting for its ledger
    /// seal.
    pub fn ledger_pending(&self) -> bool {
        self.ledger_pending
    }

    /// Whether day 100 has completed.
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Whether the protocol has been begun again at
    /// least once.
    pub fn is_looped(&self) -> bool {
        self.looped
    }

    /// The player settings.
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// The current day's unlock date. Invariant: a
    /// started state always carries one — transitions
    /// set it and the loader refuses a started save
    /// without it.
    fn current_unlock(&self) -> CalendarDate {
        assert!(self.day >= 1, "unlock date read before Permission");
        self.unlock_date
            .expect("a started state carries its unlock date")
    }

    /// Keep the stored block/layer in step with the day.
    fn sync_derived(&mut self) {
        if self.day == 0 {
            self.block = 0;
            self.layer = 0;
        } else {
            self.block = u8::try_from((self.day - 1) / 10 + 1).unwrap_or(0);
            self.layer = layer_for_day(self.day).unwrap_or(0);
        }
    }

    /// Grant Permission (lesson 1's act, design section
    /// 2): day 0 -> day 1, unlocked on `on`. The release
    /// word itself installs when day 1's Deep session
    /// completes, like every cue.
    pub fn grant_permission(&mut self, on: CalendarDate) -> Result<(), FallError> {
        if self.day != 0 {
            return Err(FallError::AlreadyStarted);
        }
        self.day = 1;
        self.sync_derived();
        self.unlock_date = Some(on);
        Ok(())
    }

    /// Beat 1 — the anchor check. `cue_lesson` names the
    /// installed cue being answered, or `None` for the
    /// spiral alone (day 1, before any cue exists). The
    /// pacing gate lives here: a new day's beats cannot
    /// start before its unlock date.
    pub fn record_anchor_check(
        &mut self,
        cue_lesson: Option<u8>,
        on: CalendarDate,
    ) -> Result<(), FallError> {
        if self.day == 0 {
            return Err(FallError::NotStarted);
        }
        if self.finished {
            return Err(FallError::ProtocolFinished);
        }
        if self.ledger_pending {
            return Err(FallError::BeatOutOfOrder { beat: "ledger" });
        }
        if self.anchor_done {
            return Err(FallError::BeatAlreadyDone { beat: "anchor" });
        }
        let unlock = self.current_unlock();
        if on < unlock {
            return Err(FallError::DayLocked {
                day: self.day,
                unlock_date: unlock,
                attempted: on,
            });
        }
        if let Some(lesson) = cue_lesson {
            if !(1..=10).contains(&lesson) {
                return Err(FallError::UnknownLesson { lesson });
            }
            if !self.has_anchor(lesson) {
                return Err(FallError::CueNotInstalled { lesson });
            }
        }
        self.anchor_done = true;
        Ok(())
    }

    /// Beat 2 — the layer's day activity. Pays the
    /// layer's activity gain into its meter. Layer 6 has
    /// no activity (design section 5).
    pub fn record_activity(&mut self, protocol: &Protocol) -> Result<(), FallError> {
        if self.day == 0 {
            return Err(FallError::NotStarted);
        }
        if self.finished {
            return Err(FallError::ProtocolFinished);
        }
        if self.ledger_pending {
            return Err(FallError::BeatOutOfOrder { beat: "ledger" });
        }
        if !self.anchor_done {
            return Err(FallError::BeatOutOfOrder { beat: "anchor" });
        }
        if self.activity_done {
            return Err(FallError::BeatAlreadyDone { beat: "activity" });
        }
        let layer = layer_for_day(self.day).unwrap_or(0);
        if layer == 6 {
            return Err(FallError::NoActivity { layer });
        }
        let row = protocol.layer_row(layer)?;
        let (meter, gain) = (row.meter, row.activity_gain);
        self.meters.add(meter, gain);
        self.activity_done = true;
        Ok(())
    }

    /// Beat 3 — a session completion. If (lesson, kind)
    /// is the current day's scheduled session, the day
    /// closes (see the module docs); otherwise the call
    /// is a free replay of an installed lesson, which
    /// never advances the day or the clock.
    pub fn complete_session(
        &mut self,
        protocol: &Protocol,
        lesson: u8,
        kind: SessionKind,
        on: CalendarDate,
    ) -> Result<SessionEffect, FallError> {
        if self.day == 0 {
            return Err(FallError::NotStarted);
        }
        if self.finished {
            return Err(FallError::ProtocolFinished);
        }
        if !(1..=10).contains(&lesson) {
            return Err(FallError::UnknownLesson { lesson });
        }
        let row = protocol.day_row(self.day)?;
        if lesson == row.lesson && kind == row.session {
            self.complete_scheduled(protocol, lesson, kind, on)
        } else {
            self.complete_replay(protocol, lesson, kind)
        }
    }

    /// The scheduled path of [`State::complete_session`]:
    /// all checks first, then the day closes in one
    /// commit.
    fn complete_scheduled(
        &mut self,
        protocol: &Protocol,
        lesson: u8,
        kind: SessionKind,
        on: CalendarDate,
    ) -> Result<SessionEffect, FallError> {
        if self.ledger_pending {
            return Err(FallError::BeatOutOfOrder { beat: "ledger" });
        }
        if !self.anchor_done {
            return Err(FallError::BeatOutOfOrder { beat: "anchor" });
        }
        if self.layer != 6 && !self.activity_done {
            return Err(FallError::BeatOutOfOrder { beat: "activity" });
        }
        let unlock = self.current_unlock();
        if on < unlock {
            return Err(FallError::DayLocked {
                day: self.day,
                unlock_date: unlock,
                attempted: on,
            });
        }
        let row = protocol.layer_row(self.layer)?;
        let (meter, gain) = if kind == SessionKind::Deep {
            (row.meter, row.deep_gain)
        } else {
            (row.meter, row.practice_gain)
        };
        // Commit: counts, cue, meter, then the day itself.
        let counts = &mut self.sessions[usize::from(lesson - 1)];
        if kind == SessionKind::Deep {
            counts.deep = counts.deep.saturating_add(1);
            self.anchors |= 1u16 << (lesson - 1);
        } else {
            counts.practice = counts.practice.saturating_add(1);
        }
        self.meters.add(meter, gain);
        if self.day == 100 {
            self.finished = true;
        } else {
            self.day += 1;
            self.sync_derived();
            // The next day unlocks on the following
            // calendar date (D-1140). At the calendar's
            // top edge the date saturates instead.
            self.unlock_date = Some(on.next_day().unwrap_or(on));
        }
        self.anchor_done = false;
        self.activity_done = false;
        self.ledger_pending = true;
        Ok(SessionEffect::Scheduled)
    }

    /// The replay path of [`State::complete_session`]:
    /// counts and the lesson's layer meter move; the
    /// day, the clock, and the beats do not.
    fn complete_replay(
        &mut self,
        protocol: &Protocol,
        lesson: u8,
        kind: SessionKind,
    ) -> Result<SessionEffect, FallError> {
        if kind == SessionKind::Deep {
            return Err(FallError::DeepNotScheduled { lesson });
        }
        if !self.has_anchor(lesson) {
            return Err(FallError::AnchorNotInstalled { lesson });
        }
        if kind == SessionKind::Maintenance && lesson < MAINTENANCE_FIRST_LESSON {
            return Err(FallError::MaintenanceNotAvailable { lesson });
        }
        let row = protocol.layer_row(layer_of_lesson(lesson))?;
        let (meter, gain) = (row.meter, row.practice_gain);
        self.sessions[usize::from(lesson - 1)].practice = self.sessions[usize::from(lesson - 1)]
            .practice
            .saturating_add(1);
        self.meters.add(meter, gain);
        Ok(SessionEffect::Replay)
    }

    /// Beat 4 — the ledger seal for the day the session
    /// beat just closed. The next day's beats cannot
    /// start before it.
    pub fn record_ledger(&mut self) -> Result<(), FallError> {
        if self.day == 0 {
            return Err(FallError::NotStarted);
        }
        if !self.ledger_pending {
            return Err(FallError::BeatOutOfOrder { beat: "ledger" });
        }
        self.ledger_pending = false;
        Ok(())
    }

    /// A mantra attempt in Piper's layer: the heard text
    /// is compared against the line after the documented
    /// normalization. The third consecutive failure on a
    /// line records as assisted — the keeper speaks the
    /// line and the day proceeds (design section 5).
    /// Attempt outcomes append to the bounded history.
    pub fn record_mantra_attempt(
        &mut self,
        protocol: &Protocol,
        line_id: &str,
        heard_text: &str,
    ) -> Result<MantraOutcome, FallError> {
        if self.day == 0 {
            return Err(FallError::NotStarted);
        }
        if layer_for_day(self.day) != Some(3) {
            return Err(FallError::MantraOutsideLayer);
        }
        if heard_text.chars().count() > MANTRA_TEXT_LEN_MAX {
            return Err(FallError::Invalid {
                field: "mantra_text",
                reason: "heard text is over its bound",
            });
        }
        let line = protocol
            .mantra_line(line_id)
            .ok_or_else(|| FallError::UnknownMantraLine {
                id: line_id.to_string(),
            })?;
        let outcome = if normalize_mantra(heard_text) == normalize_mantra(&line.text) {
            MantraOutcome::Pass
        } else {
            let streak = self
                .mantra_history
                .iter()
                .filter(|record| record.line == line_id)
                .rev()
                .take_while(|record| record.outcome == MantraOutcome::Fail)
                .count();
            if streak + 1 >= MANTRA_ATTEMPTS_FOR_ASSIST {
                MantraOutcome::Assisted
            } else {
                MantraOutcome::Fail
            }
        };
        let row = protocol.layer_row(3)?;
        let gain = match outcome {
            MantraOutcome::Pass => row.mantra_pass_gain,
            MantraOutcome::Assisted => row.mantra_assisted_gain,
            MantraOutcome::Fail => 0,
        };
        let meter = row.meter;
        self.mantra_history.push(MantraRecord {
            line: line_id.to_string(),
            outcome,
        });
        while self.mantra_history.len() > MANTRA_HISTORY_MAX {
            self.mantra_history.remove(0);
        }
        if outcome == MantraOutcome::Pass {
            self.mantra_passed_unassisted = true;
        }
        self.meters.add(meter, gain);
        Ok(outcome)
    }

    /// Whether a layer's exit is met (design section 5):
    /// the layer's final lesson's Deep session complete,
    /// its meter at threshold, and — for Piper's layer —
    /// an unassisted mantra pass on record. Layer 6's
    /// exit is the protocol's completion.
    pub fn layer_exit_met(&self, protocol: &Protocol, layer: u8) -> Result<bool, FallError> {
        if !(1..=6).contains(&layer) {
            return Err(FallError::UnknownLayer { layer });
        }
        if layer == 6 {
            return Ok(self.finished);
        }
        let row = protocol.layer_row(layer)?;
        let final_lesson = protocol.day_row(row.last_day)?.lesson;
        Ok(self.has_anchor(final_lesson)
            && self.meters.get(row.meter) >= row.exit_threshold
            && (!row.exit_mantra_required || self.mantra_passed_unassisted))
    }

    /// Take the loop-back at day 100's completion: the
    /// protocol restarts at day 1 — willingly (design
    /// section 5). Progress resets in full; settings are
    /// the player's and carry over.
    pub fn begin_again(&mut self, on: CalendarDate) -> Result<(), FallError> {
        if !self.finished {
            return Err(FallError::LoopNotOffered);
        }
        let settings = self.settings.clone();
        *self = State::new();
        self.day = 1;
        self.sync_derived();
        self.unlock_date = Some(on);
        self.looped = true;
        self.settings = settings;
        Ok(())
    }

    /// Replace the player settings, validated.
    pub fn update_settings(&mut self, settings: Settings) -> Result<(), FallError> {
        settings.validate()?;
        self.settings = settings;
        Ok(())
    }

    /// Serialize the save deterministically: same state,
    /// same bytes, on every platform (the S7 discipline).
    pub fn to_ron(&self) -> Result<String, FallError> {
        ron::ser::to_string(self).map_err(|_| FallError::SaveInvalid {
            field: "save",
            reason: "the save record failed to serialize",
        })
    }

    /// Load a save, refusing anything outside the
    /// section-4 vocabulary: unknown fields, a future
    /// format version, out-of-range values, a stored
    /// block/layer that disagrees with the day, an
    /// invalid stored date, an over-long mantra history.
    pub fn from_ron(text: &str) -> Result<State, FallError> {
        let state: State = ron::from_str(text).map_err(FallError::SaveParse)?;
        state.validate_save()?;
        Ok(state)
    }

    /// The save-shape checks `from_ron` enforces after
    /// parsing.
    fn validate_save(&self) -> Result<(), FallError> {
        if self.format_version != SAVE_FORMAT_VERSION {
            return Err(FallError::SaveInvalid {
                field: "format_version",
                reason: "the save format version is not this crate's version",
            });
        }
        if self.day > 100 {
            return Err(FallError::SaveInvalid {
                field: "day",
                reason: "day outside 0..=100",
            });
        }
        let (expected_block, expected_layer) = if self.day == 0 {
            (0, 0)
        } else {
            (
                u8::try_from((self.day - 1) / 10 + 1).unwrap_or(0),
                layer_for_day(self.day).unwrap_or(0),
            )
        };
        if expected_block != self.block || expected_layer != self.layer {
            return Err(FallError::SaveInvalid {
                field: "block/layer",
                reason: "the stored block/layer disagree with the day",
            });
        }
        if self.anchors & !0b11_1111_1111 != 0 {
            return Err(FallError::SaveInvalid {
                field: "anchors",
                reason: "anchor bits set beyond the ten cues",
            });
        }
        let meters = [
            self.meters.depth,
            self.meters.craving,
            self.meters.yes,
            self.meters.pairing,
            self.meters.ritual,
            self.meters.spiral,
        ];
        if meters.iter().any(|value| *value > METER_MAX) {
            return Err(FallError::SaveInvalid {
                field: "meters",
                reason: "a meter exceeds the cap",
            });
        }
        if self.mantra_history.len() > MANTRA_HISTORY_MAX {
            return Err(FallError::SaveInvalid {
                field: "mantra_history",
                reason: "the mantra history exceeds its bound",
            });
        }
        for record in &self.mantra_history {
            if record.line.is_empty() || record.line.chars().count() > MANTRA_ID_LEN_MAX {
                return Err(FallError::SaveInvalid {
                    field: "mantra_history",
                    reason: "a mantra record's line id is empty or over its bound",
                });
            }
        }
        if self.day == 0 && self.unlock_date.is_some() {
            return Err(FallError::SaveInvalid {
                field: "unlock_date",
                reason: "an unstarted state carries an unlock date",
            });
        }
        if self.day >= 1 && self.unlock_date.is_none() {
            return Err(FallError::SaveInvalid {
                field: "unlock_date",
                reason: "a started state has no unlock date",
            });
        }
        if self.day == 0
            && (self.anchor_done || self.activity_done || self.ledger_pending || self.finished)
        {
            return Err(FallError::SaveInvalid {
                field: "beats",
                reason: "an unstarted state carries day-loop progress",
            });
        }
        if self.ledger_pending && (self.anchor_done || self.activity_done) {
            return Err(FallError::SaveInvalid {
                field: "beats",
                reason: "a pending ledger cannot coexist with today's beats",
            });
        }
        if self.finished && self.day != 100 {
            return Err(FallError::SaveInvalid {
                field: "finished",
                reason: "the protocol can only finish on day 100",
            });
        }
        self.settings
            .validate()
            .map_err(|_| FallError::SaveInvalid {
                field: "settings",
                reason: "the settings break their bounds",
            })?;
        Ok(())
    }
}
