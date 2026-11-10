//! torogao-fall: the Where She Asks to Fall protocol as
//! Torogao data + systems.
//!
//! Clean-room provenance. The game is a premise-only
//! remake of "100 Days of Self Hypnosis" (Fadai Games):
//! at its S2c screen the original was EXCLUDED as a
//! content source (character ages unstated in every
//! record), so nothing of it — character, script, art,
//! audio — is extracted, studied beyond its public
//! page, or shipped. What transfers is the premise (a
//! 100-day self-hypnosis training protocol) under the
//! Roseford pattern. The content core is the estate's
//! own Surrender Curriculum (D-1091/D-1092), the cast is
//! the estate's Torogao six, and every value in the data
//! file is a DESIGN value from the record of record:
//! `games/where-she-asks-to-fall-design.md` in the Lane
//! D programme tree (`nsfw-remakes-20261108/`), under
//! D-1140.
//!
//! The crate is a systems skeleton for the Torogao
//! remake programme (PIPELINE.md in
//! `nsfw-remakes-20261108/`): pure, deterministic
//! functions over explicit inputs. No wall clock, no
//! RNG, no I/O beyond string (de)serialization. Dates
//! enter as values; the pacing clock is data, not a
//! system call. Shell binding (Torogao session UI, STT,
//! audio playback), the animation restaging, and the
//! dual-target S7 hash proof are later stages, not this
//! crate.

#![forbid(unsafe_code)]

pub mod date;
pub mod protocol;
pub mod state;
pub mod trajectory;

pub use date::CalendarDate;
pub use protocol::{
    CueRow, DAY_COUNT, DayRow, FallError, LAYER_COUNT, LAYER_RANGES, LESSON_COUNT, LayerRow,
    METER_MAX, MantraRow, MeterKind, PROTOCOL_RON, PROTOCOL_SCHEMA, PairingRow, Protocol,
    SessionKind, design_protocol, layer_for_day,
};
pub use state::{
    LessonCounts, MAINTENANCE_FIRST_LESSON, MANTRA_ATTEMPTS_FOR_ASSIST, MANTRA_HISTORY_MAX,
    MantraOutcome, MantraRecord, Meters, SAVE_FORMAT_VERSION, SessionEffect, Settings, SpiralStyle,
    State, SurfacePosition,
};
pub use trajectory::{FixtureRun, hash_state, run_design_fixture, run_fixture};
