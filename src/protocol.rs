//! The protocol tables for Where She Asks to Fall, as
//! data.
//!
//! The tables ship as RON (`data/protocol.ron`, schema
//! `torogao-fall-protocol/1`) following the estate's RON
//! contract discipline: the file is the contract, this
//! module validates it at load against the design doc's
//! structural shape, and any value outside that shape is
//! a typed refusal — never a silent default.
//!
//! Every value is a DESIGN value from
//! `games/where-she-asks-to-fall-design.md` (D-1140): the
//! studied original was excluded as a content source at
//! its S2c screen, so nothing here is measured from it.
//! Meter increments live in the layer table (data), not
//! in code, per the design doc's section 8.

use serde::Deserialize;

use crate::date::CalendarDate;

/// The embedded protocol tables (see the file header for
/// provenance and the design sections behind every value).
pub const PROTOCOL_RON: &str = include_str!("../data/protocol.ron");

/// The only schema this loader accepts.
pub const PROTOCOL_SCHEMA: &str = "torogao-fall-protocol/1";
/// Protocol days in a full run (design section 3).
pub const DAY_COUNT: usize = 100;
/// Blocks (and lessons) in the ladder: ten of each.
pub const LESSON_COUNT: usize = 10;
/// Layer keepers in the protocol.
pub const LAYER_COUNT: usize = 6;
/// Meters run 0..=METER_MAX (design section 4).
pub const METER_MAX: u32 = 1000;
/// The mantra table carries at least this many lines
/// (the task of record for the S4 build).
pub const MANTRA_LINE_MIN: usize = 4;
/// A bound on any keeper/title/cue string in the data
/// file, so a corrupt file cannot allocate unbounded
/// text. The design strings run well under it.
pub const NAME_LEN_MAX: usize = 128;
/// A bound on one mantra line's text.
pub const MANTRA_TEXT_LEN_MAX: usize = 256;

/// Canonical layer day-ranges (design section 5); the
/// data file's layer table must agree with these.
pub const LAYER_RANGES: [(u16, u16); LAYER_COUNT] =
    [(1, 20), (21, 40), (41, 60), (61, 70), (71, 90), (91, 100)];

/// The layer (1..=6) a protocol day belongs to, or `None`
/// for day 0 (before Permission) and days past 100.
pub fn layer_for_day(day: u16) -> Option<u8> {
    LAYER_RANGES
        .iter()
        .position(|(first, last)| day >= *first && day <= *last)
        .map(|idx| u8::try_from(idx + 1).unwrap_or(0))
}

/// One of the six layer meters (design section 4).
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum MeterKind {
    Depth,
    Craving,
    Yes,
    Pairing,
    Ritual,
    Spiral,
}

/// The kind of a session (design section 3). Deep is a
/// block's anchor session (the Full edition); Practice is
/// a day session or free replay (Lite); Maintenance is a
/// replay-only variant, native from lesson 5.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum SessionKind {
    Deep,
    Practice,
    Maintenance,
}

/// One row of the 100-day table: what session a protocol
/// day schedules.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DayRow {
    /// Protocol day, 1..=100.
    pub day: u16,
    /// Block, 1..=10 — always the day's lesson (block N is
    /// anchored by lesson N).
    pub block: u8,
    /// The lesson this day's session belongs to.
    pub lesson: u8,
    /// Deep on a block's first day, Practice otherwise.
    /// Maintenance is never scheduled.
    pub session: SessionKind,
}

/// One row of the layer table: a keeper, her meter, her
/// exit threshold, and the meter gains her sessions and
/// drills pay (the tuning lives here, not in code).
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LayerRow {
    /// Layer, 1..=6.
    pub layer: u8,
    /// The keeper's name (estate cast).
    pub keeper: String,
    /// The layer's title (design section 5).
    pub title: String,
    /// The meter this layer advances.
    pub meter: MeterKind,
    /// First protocol day of the layer.
    pub first_day: u16,
    /// Last protocol day of the layer.
    pub last_day: u16,
    /// Meter value the exit requires (0 = no meter gate;
    /// layer 6 completes by finishing day 100 instead).
    pub exit_threshold: u32,
    /// Whether the exit also requires an unassisted
    /// mantra pass (Piper's layer only).
    pub exit_mantra_required: bool,
    /// Meter paid by a completed Deep session.
    pub deep_gain: u32,
    /// Meter paid by a completed Practice or Maintenance
    /// session, scheduled or replayed.
    pub practice_gain: u32,
    /// Meter paid by the layer's day activity.
    pub activity_gain: u32,
    /// Meter paid by an unassisted mantra pass (layer 3).
    pub mantra_pass_gain: u32,
    /// Meter paid by an assisted mantra pass (layer 3).
    pub mantra_assisted_gain: u32,
}

/// One row of the cue table: the anchor a lesson installs
/// when its Deep session completes.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CueRow {
    /// Lesson, 1..=10.
    pub lesson: u8,
    /// The lesson's title (The Surrender Curriculum).
    pub title: String,
    /// The installed cue's name (design section 4, in
    /// ladder order).
    pub cue: String,
}

/// One mantra line for Piper's layer: an estate-original
/// protocol affirmation the speech check verifies.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MantraRow {
    /// Stable line id (e.g. "mantra-1").
    pub id: String,
    /// The expected spoken text.
    pub text: String,
}

/// One step of Sloane's pairing schedule: sensation
/// prompts tick at `tick_seconds`, at `intensity_percent`
/// of the paired channel's ceiling. Data for the shell;
/// the core only validates and carries it.
#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PairingRow {
    /// Step number, 1-based and sequential.
    pub step: u8,
    /// Seconds between sensation ticks at this step.
    pub tick_seconds: u32,
    /// Intensity, 0..=100 percent.
    pub intensity_percent: u8,
}

/// The data file's root shape, before validation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProtocolFile {
    schema: String,
    days: Vec<DayRow>,
    layers: Vec<LayerRow>,
    cues: Vec<CueRow>,
    mantra_lines: Vec<MantraRow>,
    pairing_schedule: Vec<PairingRow>,
}

/// Every refusal the crate can return. Data and save
/// problems are typed errors with bounded context;
/// transition refusals name the rule that fired.
#[derive(Debug)]
pub enum FallError {
    /// The protocol RON did not parse as the schema (this
    /// includes unknown enum spellings and unknown fields:
    /// deserialization is deny-by-default).
    Parse(ron::de::SpannedError),
    /// A parsed protocol value fell outside the design
    /// shape.
    Invalid {
        field: &'static str,
        reason: &'static str,
    },
    /// The save RON did not parse as the vocabulary.
    SaveParse(ron::de::SpannedError),
    /// A parsed save fell outside the section-4
    /// vocabulary (including a future format version).
    SaveInvalid {
        field: &'static str,
        reason: &'static str,
    },
    /// A calendar date that does not exist.
    InvalidDate { year: i32, month: u32, day: u32 },
    /// A day-loop beat attempted before Permission.
    NotStarted,
    /// Permission granted twice.
    AlreadyStarted,
    /// A beat or scheduled session attempted after day
    /// 100 completed; the only way forward is begin-again.
    ProtocolFinished,
    /// A beat attempted on a date before the current
    /// day's unlock date (calendar pacing, D-1140).
    DayLocked {
        day: u16,
        unlock_date: CalendarDate,
        attempted: CalendarDate,
    },
    /// A Deep session that is not the current day's
    /// scheduled session (Deep is never a replay).
    DeepNotScheduled { lesson: u8 },
    /// A replay attempted for a lesson whose anchor is
    /// not installed yet (its Deep session has not
    /// completed).
    AnchorNotInstalled { lesson: u8 },
    /// A Maintenance replay for a lesson below 5
    /// (maintenance editions are native from lesson 5).
    MaintenanceNotAvailable { lesson: u8 },
    /// A lesson outside 1..=10.
    UnknownLesson { lesson: u8 },
    /// A layer outside 1..=6.
    UnknownLayer { layer: u8 },
    /// An anchor check naming a cue that is not
    /// installed.
    CueNotInstalled { lesson: u8 },
    /// A mantra attempt naming a line the table does
    /// not carry.
    UnknownMantraLine { id: String },
    /// A mantra attempt outside Piper's layer (the
    /// checks live in layer 3 only).
    MantraOutsideLayer,
    /// A day activity attempted in a layer that has
    /// none (layer 6: sessions and anchors only).
    NoActivity { layer: u8 },
    /// A beat attempted before its predecessor in the
    /// day loop (anchor -> activity -> session ->
    /// ledger).
    BeatOutOfOrder { beat: &'static str },
    /// A beat attempted twice in one day.
    BeatAlreadyDone { beat: &'static str },
    /// Begin-again taken before the protocol completed.
    LoopNotOffered,
    /// Settings that break their bounds (e.g. volume
    /// above 100 percent).
    InvalidSettings { reason: &'static str },
}

impl std::fmt::Display for FallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FallError::Parse(e) => write!(f, "protocol RON parse failed: {e}"),
            FallError::Invalid { field, reason } => {
                write!(f, "protocol field {field}: {reason}")
            }
            FallError::SaveParse(e) => write!(f, "save RON parse failed: {e}"),
            FallError::SaveInvalid { field, reason } => {
                write!(f, "save field {field}: {reason}")
            }
            FallError::InvalidDate { year, month, day } => {
                write!(f, "no such calendar date: {year}-{month:02}-{day:02}")
            }
            FallError::NotStarted => write!(f, "the protocol has not been granted"),
            FallError::AlreadyStarted => write!(f, "permission was already granted"),
            FallError::ProtocolFinished => {
                write!(f, "the protocol is complete; begin again to continue")
            }
            FallError::DayLocked {
                day,
                unlock_date,
                attempted,
            } => write!(
                f,
                "day {day} unlocks on {:04}-{:02}-{:02}; attempted {:04}-{:02}-{:02}",
                unlock_date.year(),
                unlock_date.month(),
                unlock_date.day(),
                attempted.year(),
                attempted.month(),
                attempted.day()
            ),
            FallError::DeepNotScheduled { lesson } => {
                write!(f, "lesson {lesson}'s deep session is not scheduled today")
            }
            FallError::AnchorNotInstalled { lesson } => {
                write!(f, "lesson {lesson}'s anchor is not installed")
            }
            FallError::MaintenanceNotAvailable { lesson } => {
                write!(f, "lesson {lesson} has no maintenance edition")
            }
            FallError::UnknownLesson { lesson } => write!(f, "no lesson {lesson}"),
            FallError::UnknownLayer { layer } => write!(f, "no layer {layer}"),
            FallError::CueNotInstalled { lesson } => {
                write!(f, "lesson {lesson}'s cue is not installed")
            }
            FallError::UnknownMantraLine { id } => write!(f, "no mantra line {id}"),
            FallError::MantraOutsideLayer => {
                write!(f, "mantra checks live in layer 3 only")
            }
            FallError::NoActivity { layer } => write!(f, "layer {layer} has no day activity"),
            FallError::BeatOutOfOrder { beat } => {
                write!(f, "the {beat} beat is out of order in the day loop")
            }
            FallError::BeatAlreadyDone { beat } => {
                write!(f, "the {beat} beat already happened today")
            }
            FallError::LoopNotOffered => {
                write!(f, "begin-again is offered only at day 100's completion")
            }
            FallError::InvalidSettings { reason } => write!(f, "settings: {reason}"),
        }
    }
}

impl std::error::Error for FallError {}

/// The validated protocol tables.
#[derive(Clone, Debug, PartialEq)]
pub struct Protocol {
    days: Vec<DayRow>,
    layers: Vec<LayerRow>,
    cues: Vec<CueRow>,
    mantra_lines: Vec<MantraRow>,
    pairing_schedule: Vec<PairingRow>,
}

impl Protocol {
    /// Load and validate protocol tables from RON text.
    /// Beyond parsing, the loader cross-checks the file
    /// against the design's structure: exactly 100 days,
    /// blocks and lessons aligned with the day derivation,
    /// layer ranges canonical, one cue per lesson.
    pub fn load(ron_text: &str) -> Result<Protocol, FallError> {
        let file: ProtocolFile = ron::from_str(ron_text).map_err(FallError::Parse)?;
        if file.schema != PROTOCOL_SCHEMA {
            return Err(FallError::Invalid {
                field: "schema",
                reason: "schema string does not match torogao-fall-protocol/1",
            });
        }
        validate_days(&file.days)?;
        validate_layers(&file.layers)?;
        validate_cues(&file.cues)?;
        validate_mantra(&file.mantra_lines)?;
        validate_pairing(&file.pairing_schedule)?;
        Ok(Protocol {
            days: file.days,
            layers: file.layers,
            cues: file.cues,
            mantra_lines: file.mantra_lines,
            pairing_schedule: file.pairing_schedule,
        })
    }

    /// The row for protocol day `day` (1..=100).
    pub fn day_row(&self, day: u16) -> Result<&DayRow, FallError> {
        if day < 1 || usize::from(day) > DAY_COUNT {
            return Err(FallError::Invalid {
                field: "day",
                reason: "day outside 1..=100",
            });
        }
        Ok(&self.days[usize::from(day - 1)])
    }

    /// All 100 day rows, in day order.
    pub fn days(&self) -> &[DayRow] {
        &self.days
    }

    /// The row for layer `layer` (1..=6).
    pub fn layer_row(&self, layer: u8) -> Result<&LayerRow, FallError> {
        if layer < 1 || usize::from(layer) > LAYER_COUNT {
            return Err(FallError::UnknownLayer { layer });
        }
        Ok(&self.layers[usize::from(layer - 1)])
    }

    /// All six layer rows, in layer order.
    pub fn layers(&self) -> &[LayerRow] {
        &self.layers
    }

    /// The cue row for lesson `lesson` (1..=10).
    pub fn cue(&self, lesson: u8) -> Result<&CueRow, FallError> {
        if lesson < 1 || usize::from(lesson) > LESSON_COUNT {
            return Err(FallError::UnknownLesson { lesson });
        }
        Ok(&self.cues[usize::from(lesson - 1)])
    }

    /// All ten cue rows, in lesson order.
    pub fn cues(&self) -> &[CueRow] {
        &self.cues
    }

    /// A mantra line by id, if the table carries it.
    pub fn mantra_line(&self, id: &str) -> Option<&MantraRow> {
        self.mantra_lines.iter().find(|row| row.id == id)
    }

    /// All mantra lines.
    pub fn mantra_lines(&self) -> &[MantraRow] {
        &self.mantra_lines
    }

    /// Sloane's pairing schedule, in step order.
    pub fn pairing_schedule(&self) -> &[PairingRow] {
        &self.pairing_schedule
    }
}

/// The design protocol tables, embedded in the crate.
pub fn design_protocol() -> Result<Protocol, FallError> {
    Protocol::load(PROTOCOL_RON)
}

/// A bounded-text check shared by the table validators.
fn check_text(field: &'static str, text: &str, max: usize) -> Result<(), FallError> {
    if text.is_empty() || text.chars().count() > max {
        return Err(FallError::Invalid {
            field,
            reason: "text is empty or over its bound",
        });
    }
    Ok(())
}

fn validate_days(days: &[DayRow]) -> Result<(), FallError> {
    if days.len() != DAY_COUNT {
        return Err(FallError::Invalid {
            field: "days",
            reason: "the day table must carry exactly 100 rows",
        });
    }
    for (idx, row) in days.iter().enumerate() {
        let day = u16::try_from(idx + 1).unwrap_or(0);
        let block = u8::try_from((idx / 10) + 1).unwrap_or(0);
        if row.day != day || row.block != block || row.lesson != block {
            return Err(FallError::Invalid {
                field: "days",
                reason: "a day row disagrees with the day/block/lesson derivation",
            });
        }
        let expected = if day % 10 == 1 {
            SessionKind::Deep
        } else {
            SessionKind::Practice
        };
        if row.session != expected {
            return Err(FallError::Invalid {
                field: "days",
                reason: "a day row's session kind disagrees with the block shape",
            });
        }
    }
    Ok(())
}

fn validate_layers(layers: &[LayerRow]) -> Result<(), FallError> {
    if layers.len() != LAYER_COUNT {
        return Err(FallError::Invalid {
            field: "layers",
            reason: "the layer table must carry exactly six rows",
        });
    }
    let meters = [
        MeterKind::Depth,
        MeterKind::Craving,
        MeterKind::Yes,
        MeterKind::Pairing,
        MeterKind::Ritual,
        MeterKind::Spiral,
    ];
    for (idx, row) in layers.iter().enumerate() {
        let layer = u8::try_from(idx + 1).unwrap_or(0);
        if row.layer != layer
            || (row.first_day, row.last_day) != LAYER_RANGES[idx]
            || row.meter != meters[idx]
        {
            return Err(FallError::Invalid {
                field: "layers",
                reason: "a layer row disagrees with the canonical ranges or meters",
            });
        }
        if row.exit_threshold > METER_MAX
            || row.deep_gain > METER_MAX
            || row.practice_gain > METER_MAX
            || row.activity_gain > METER_MAX
            || row.mantra_pass_gain > METER_MAX
            || row.mantra_assisted_gain > METER_MAX
        {
            return Err(FallError::Invalid {
                field: "layers",
                reason: "a threshold or gain exceeds the meter cap",
            });
        }
        check_text("layers.keeper", &row.keeper, NAME_LEN_MAX)?;
        check_text("layers.title", &row.title, NAME_LEN_MAX)?;
    }
    Ok(())
}

fn validate_cues(cues: &[CueRow]) -> Result<(), FallError> {
    if cues.len() != LESSON_COUNT {
        return Err(FallError::Invalid {
            field: "cues",
            reason: "the cue table must carry exactly ten rows",
        });
    }
    for (idx, row) in cues.iter().enumerate() {
        let lesson = u8::try_from(idx + 1).unwrap_or(0);
        if row.lesson != lesson {
            return Err(FallError::Invalid {
                field: "cues",
                reason: "cue rows must be in lesson order",
            });
        }
        check_text("cues.title", &row.title, NAME_LEN_MAX)?;
        check_text("cues.cue", &row.cue, NAME_LEN_MAX)?;
    }
    Ok(())
}

fn validate_mantra(lines: &[MantraRow]) -> Result<(), FallError> {
    if lines.len() < MANTRA_LINE_MIN {
        return Err(FallError::Invalid {
            field: "mantra_lines",
            reason: "the mantra table carries fewer lines than the minimum",
        });
    }
    for (idx, row) in lines.iter().enumerate() {
        check_text("mantra_lines.id", &row.id, NAME_LEN_MAX)?;
        check_text("mantra_lines.text", &row.text, MANTRA_TEXT_LEN_MAX)?;
        if lines[..idx].iter().any(|earlier| earlier.id == row.id) {
            return Err(FallError::Invalid {
                field: "mantra_lines",
                reason: "duplicate mantra line id",
            });
        }
    }
    Ok(())
}

fn validate_pairing(schedule: &[PairingRow]) -> Result<(), FallError> {
    if schedule.is_empty() {
        return Err(FallError::Invalid {
            field: "pairing_schedule",
            reason: "the pairing schedule is empty",
        });
    }
    for (idx, row) in schedule.iter().enumerate() {
        let step = u8::try_from(idx + 1).unwrap_or(0);
        if row.step != step || row.tick_seconds < 1 || row.intensity_percent > 100 {
            return Err(FallError::Invalid {
                field: "pairing_schedule",
                reason: "pairing steps must be sequential with sane tick and intensity",
            });
        }
    }
    Ok(())
}
