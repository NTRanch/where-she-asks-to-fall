# torogao-fall — S4 build report (Lane D QUEUE #27)

Built 2026-11-09 under the Owner's S4 commission (D-1140
programme). Spec of record:
`games/where-she-asks-to-fall-design.md`. The crate
implements the design doc's §3 day loop, §4 state
vocabulary, §5 layer exits, and §8 systems shape as a
pure systems core: no clock, filesystem, or environment
access inside the core — dates and data enter as values.

## Landing

- Tree (primo): `~/.local/share/pax/fall-s4-20261109/wt`
  (git worktree of the estate Torogao repo, the same
  arrangement as the timestop/milkfactory S4 lanes).
- Branch: `pax/fall-s4-20261109`, based on **68706e2** —
  the base both timestop (d4cbec0) and milkfactory
  (8a19b41) branched from (it carries coin 355d213 and
  modapp 6894a4b). One commit, **not pushed**:
  - `d3d30a9` torogao-fall: clean-room Where She Asks
    to Fall systems crate (Lane D queue #27, S4)
- Commit contents: root `Cargo.toml` (one members
  line), `Cargo.lock` (the new member), and
  `crates/torogao-fall/` (Cargo.toml, data/protocol.ron,
  src/{lib,main,date,protocol,state,trajectory}.rs,
  tests/fall_tests.rs). Working tree clean after the
  commit; the worktree's `target/` was removed at task
  end, matching the sibling worktrees.

## API summary

- `Protocol::load(&str)` / `design_protocol()` — the
  RON tables (`data/protocol.ron`, schema
  `torogao-fall-protocol/1`, embedded via
  `include_str!`). Every struct is
  `deny_unknown_fields`; the loader cross-validates:
  exactly 100 day rows aligned with the day/block/
  lesson derivation (Deep iff block day 1), six layer
  rows matching the canonical ranges and meter order,
  ten cue rows in lesson order, ≥4 mantra lines with
  unique ids, a sequential pairing schedule, all
  thresholds/gains ≤ the 1000 meter cap, bounded text.
- `CalendarDate` — validated civil date (see below).
- `State` — the §4 vocabulary and the whole save file:
  day, stored block/layer (verified against the
  derivation at load), the 10-bit anchor mask, six
  meters, per-lesson (deep, practice) counts, the
  mantra history (last 8) plus a cumulative
  unassisted-pass flag, the pacing unlock date, the
  day's beat flags, finished/looped flags, settings.
  `to_ron`/`from_ron` with `format_version` (future
  versions refused).
- Transitions (validate fully, then commit; a refusal
  leaves the state byte-identical):
  `grant_permission(date)`,
  `record_anchor_check(Option<lesson>, date)` (beat 1;
  carries the pacing gate),
  `record_activity(&protocol)` (beat 2; refused in
  layer 6), `complete_session(&protocol, lesson, kind,
  date)` (beat 3; scheduled vs replay — see decisions),
  `record_ledger()` (beat 4),
  `record_mantra_attempt(&protocol, line_id, heard)`
  (layer 3 only; normalization = trim + case-fold +
  whitespace collapse + one trailing period dropped;
  third consecutive failure on a line records
  `Assisted`), `layer_exit_met(&protocol, layer)`,
  `begin_again(date)`, `update_settings(settings)`.
- `run_fixture` / `run_design_fixture` + `hash_state` —
  the canonical 10-day compressed fixture (injected
  dates from 2027-03-01) and the S7 state hash: FNV-1a
  64-bit over the canonical serialized save bytes (the
  coin/timestop discipline, re-implemented in-crate).
- `fall-smoke` bin prints the fixture outcome and both
  hashes.

## Gates (primo, nice -n 10, CARGO_BUILD_JOBS=8,
nightly-2026-10-30)

- `cargo fmt -p torogao-fall -- --check` — **rc 0**
- `cargo build -p torogao-fall` — **rc 0**
- `cargo clippy -p torogao-fall --all-targets --
  -D warnings` — **rc 0**
- `cargo test -p torogao-fall` — **28 passed, 0
  failed**: 5 unit (date ×3, trajectory ×2) + 23
  integration in `tests/fall_tests.rs`, split by name
  prefix: **11 `validation_` / 12 `adversarial_`**.
- Theodolite (bridge 1.0.0, bacon-ls 0.23.0,
  diagnostics owner, session phase "ready", no
  possibly-incomplete flags): **0 diagnostics** on
  every touched file — src/lib.rs, src/main.rs,
  src/date.rs, src/protocol.rs, src/state.rs,
  src/trajectory.rs, tests/fall_tests.rs (7 files).
- Smoke: `fall-smoke: success=true days=10 steps=41
  anchors=0000000001 depth=144
  state_hash=c61148c2fb666721
  trajectory_hash=1857cd634200ec9e` (identical across
  runs, including after the fmt pass).

## Date representation

A validated civil **(year, month, day)** triple, not a
days-since-epoch count: the save stays
human-inspectable and the pacing rule is stated in
calendar terms. Construction is only through
`CalendarDate::new`, which validates against the real
proleptic Gregorian calendar (Feb 30 and Feb 29 on
non-leap years are typed refusals). Deserialization
crosses the same constructor, so a stored invalid date
is refused at load. Ordering is lexicographic =
chronological; epoch arithmetic (Hinnant's algorithm)
exists only inside `next_day()`. The pacing rule as
encoded: completing day N's scheduled session on date
T sets day N+1's unlock date to T+1; a day's beats and
its scheduled session refuse any earlier date
(`DayLocked`). One new protocol day per real date;
replays never touch the clock.

## Meter increments (data: the layer rows)

| Layer | Meter | Deep | Practice | Activity | Mantra pass/assist | Exit | Full-attendance total | Sessions-only total |
|---|---|---|---|---|---|---|---|---|
| 1 Lux | Depth | 40 | 6 | 5 | – | ≥200 | 288 | 188 |
| 2 Yui | Craving | 60 | 10 | 8 | – | ≥400 | 460 | 300 |
| 3 Piper | Yes | 80 | 14 | 12 | 15/5 | ≥600 + unassisted pass | 652 (+mantra) | 412 |
| 4 Sloane | Pairing | 200 | 30 | 25 | – | ≥700 | 720 | 470 |
| 5 Alma | Ritual | 120 | 20 | 15 | – | ≥850 | 900 | 600 |
| 6 Prima | Spiral | 300 | 40 | 0 | – | day 100 | 660 | 660 |

Full attendance = every beat every day of the layer.
By construction each threshold clears only with the
drills done — sessions alone fall short (the drills
are load-bearing, per design §5). All gains saturate
at the 1000 cap.

## Decisions taken where the design doc left room

1. **Day closure.** The day counter advances at the
   session beat (beat 3); the ledger beat then seals
   the completed day, and the next day's beats refuse
   until it lands. Beat order anchor → activity →
   session → ledger is enforced throughout.
2. **Scheduled vs replay.** A `complete_session` call
   whose (lesson, kind) equals the current day's
   table row is the scheduled session; anything else
   is judged as a replay. Consequence: block days
   2–10 *are* the block lesson's Practice sessions, so
   a same-lesson Practice call on those days is the
   day's session, not a replay. A replay of an earlier
   lesson is distinguishable (and tested) from the
   next block onward.
3. **Maintenance counting.** Maintenance completions
   increment the practice counter — §4's vocabulary
   has exactly two counters and a maintenance edition
   is a practice variant. Maintenance stays
   replay-only and gated to lessons ≥5.
4. **Cue order.** §4's parenthetical read as the
   ordered per-lesson list: 1 release word, 2 "deep
   and calm", 3 breath, 4 sound, 5 craving, 6 yes,
   7 pleasure-binding, 8 "now, for me", 9 ache,
   10 "begin again". By the uniform rule, cue 10
   installs at lesson 10's Deep completion (day 91);
   day 100's completion sets the finished flag and
   offers the loop-back (§5's "day 100 installs the
   loop-back cue" read as the offer, not a second
   install rule).
5. **Mantra assist + exit flag.** The assist streak
   is derived from the saved history (trailing
   per-line Fail run), so no extra counter state
   exists. The unassisted-pass requirement is a
   cumulative flag: deriving it from the 8-entry
   history window would let later attempts un-satisfy
   Piper's exit.
6. **Beats are saved.** `anchor_done`,
   `activity_done`, `ledger_pending` are part of the
   save — §4's closing line makes transitions pure
   functions of "the day's completed beats", and a
   mid-day restart must not strand a completed
   session's ledger seal.
7. **After completion.** Once day 100 completes,
   session/beat calls refuse (`ProtocolFinished`);
   "replay freely" is read as applying during the
   run. `begin_again` is the way forward; it resets
   all progress, preserves settings, sets `looped`,
   and unlocks the new day 1 on the begin-again date.
8. **Pairing schedule** is validated data only — the
   core has no consumer for it; it is the shell's
   Layer-4 pacing data, carried like the siblings
   carry their tables.
9. **Surface-control interruption** (design §8 test
   list) has no partial state to corrupt in the core:
   a session exists only once its completion records,
   so interruption mid-session is inherently clean.
   The core-side equivalent is the atomicity test.
10. **Calendar top edge.** If a day completed on
    9999-12-31, the next unlock date saturates to the
    same date (documented in code); the constructor
    still refuses year 10000.

## Process notes (D-713 honesty)

- The first gate run showed 20/23 integration tests
  passing. All three failures were **test-side**
  misreadings of the block structure (decision 2
  above): the tests treated a block's Practice days as
  replays of lesson 1. The core was correct in each
  case; the tests were fixed and the suite re-run to
  23/23. No core code changed after the first build.
- Theodolite: a first session sending seven
  diagnostics requests cold never returned within a
  900 s window (only the initialize response arrived).
  The warm recipe — `session_status` first, then
  per-file `diagnostics` — answered in seconds with
  the server phase "ready". Recorded so the next S4
  lane doesn't re-derive it.
