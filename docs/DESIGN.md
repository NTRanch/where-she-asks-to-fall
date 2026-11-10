# WHERE SHE ASKS TO FALL — design doc (QUEUE #27)

Status: DESIGN, landed 2026-11-09 under D-1140 and its
design note. Clean-room: premise and systems studied from
the original's public page only (see
hundred-days-surrender-intake.md for the census and the
S2c record: the original is EXCLUDED as a content source;
this game is premise-only, all content estate-original).
Every number in this doc is a DESIGN value, set by this
doc — nothing is measured from the original, which exposes
no tuning data (browser prototype, no downloads).

## 1. Identity

- Title: Where She Asks to Fall (Owner, D-1140 naming).
- Form: a 100-day brainwashing/training protocol played
  in the first person. The trainee is the user; there is
  no separate heroine. Six layer keepers (the Torogao
  cast) run the protocol.
- Content core: The Surrender Curriculum — ten lessons
  (D-1091/D-1092 spec, v2 masters with the disclosed
  SSB-AM comment suggestion, D-1135/D-1140) as the
  session audio. The series name survives inside the
  game as the protocol's own name for its lesson arc.
- Engine: Torogao (D-457). Dual-target law (D-1094):
  native Linux + Android from one systems crate + one
  data file, equal determinism hashes at the S7 gate.
- Distribution: free (D-1134), one itch game page as the
  Curriculum's sole itch home (D-1140). Page creation is
  blocked on the itch 20-project limit until the raise
  lands; L1/L2 standalone pages stay live until then.

## 2. The frame: fiction vs product

Inside the fiction this is a brainwashing protocol and
the keepers speak like it. The product's real contract
stays the title — she asks to fall:

- 18+ gate at first launch, before any content.
- L1 is Permission: the protocol's first act is the
  user granting it, in-fiction and mechanically (the
  day counter does not exist before it).
- A SURFACE control is on screen at all times during
  any session: one press stops the audio, kills the
  spiral, and lands on a plain bright screen. It is
  never hidden, never disabled, never further than
  one tap.
- Every session ends on an aftercare screen (water,
  light, orientation lines) — the series' own session
  hygiene, in the Pendulum safety-furniture tradition.
- The disclosed-suggestion posture of the audio is
  stated on the game page and in the credits screen,
  verbatim in spirit with the D-1135 disclosure.

## 3. The hundred days

- 10 blocks x 10 days. Block N is anchored by Lesson N.
- A block's day 1 is the DEEP SESSION (the lesson's
  Full edition). Days 2-10 are PRACTICE days: the
  Lite edition, maintenance variants (the series
  ships Maintenance editions from L5; earlier blocks
  derive theirs from the anchor lesson), anchor
  drills, and the layer's day activity.
- Day loop (deterministic, four beats):
  1. ANCHOR — the spiral/pendulum visual (customizable,
     per the original's feature list) + a cue-response
     check: the game speaks or shows a cue installed
     by an earlier lesson; the user's response (tap,
     or voice in Piper's layer) is the input.
  2. ACTIVITY — the layer's day activity (section 5).
  3. SESSION — the day's audio (deep or practice),
     presented by the layer's keeper (section 5).
  4. LEDGER — one line written to the protocol ledger
     (before Layer 3, a simple day seal; from Piper's
     layer on, her ledger format: what was practiced,
     what the yes answered).
- PACING — RULED (Owner, D-1140 pacing note,
  2026-11-09): calendar-paced. One new protocol day
  unlocks per real day; practice days and completed
  content replay freely without advancing the
  counter. The protocol fantasy is daily training,
  and the clock is part of the training.

## 4. State vocabulary (the whole save file)

DESIGN values; the loader refuses anything outside
this vocabulary (the coin/modapp pattern).

- day (0-100; 0 = before Permission)
- block (1-10), layer (1-6) — derived from day, stored
  for save-format stability
- anchors: 10-bit mask of installed cues, one per
  lesson (release word, "deep and calm", breath, sound,
  craving, yes, pleasure-binding, identity cue
  "now, for me", ache, loop-back "begin again") —
  cue names from the series ladder, D-1091
- meters: six per-layer values 0-1000 (Depth/Lux,
  Craving/Yui, Yes/Piper, Pairing/Sloane, Ritual/Alma,
  Spiral/Prima), advanced only by that layer's
  completed sessions and drills
- sessions: per-lesson completion counts (deep,
  practice), mantra-check results in Piper's layer
  (pass/fail per attempt, bounded history of last 8)
- pacing: the local calendar date on which the
  current day unlocked; the next day's deep session
  unlocks when the local date passes it (D-1140
  pacing note). Practice replays never move it
- settings: spiral style, pendulum on/off, volume,
  surface-control position
State transitions are pure functions of (prior state,
the day's completed beats). Same inputs -> same state
-> same S7 hash on both targets.

## 5. The six layers

Each layer: keeper, lessons/days, her conditioning
vehicle (her D-273 signature kink as hypnotic method),
her presentation, her day activity, her exit condition
(the next layer opens when the exit is met AND the
block's deep session is complete).

### Layer 1 — INDUCTION, "The Lens" — Lux
- Lessons 1-2 (days 1-20). Vehicle: being watched. The
  camera is the induction device; her recorded gaze and
  the monitor showing "you, live" pace the descent.
- Presentation: her animation inside a REC frame with
  a live-monitor inset (her keeper still's staging);
  the spiral rides the monitor behind her.
- Activity: THE POSE — hold a presented posture/pose
  for the camera across a slow timer; stillness is
  the drill (her exhibitionism as the user's focus
  training).
- Exit: L2 deep session complete + Depth >= 200.

### Layer 2 — DEEPENING, "The Rope" — Yui
- Lessons 3-4 (days 21-40). Vehicle: fractionation.
  The sound anchor drops the user repeatedly inside
  every session; her edging grammar becomes craving
  for the next drop.
- Presentation: her animation bound in the rope frame;
  each anchor cue tightens the visual composition
  (one more rope pass per installed cue this layer).
- Activity: THE WAIT — a held anticipation timer the
  user may not skip; early exit from the wait is
  allowed (surface rules) but the drill only counts
  when endured.
- Exit: L4 deep session complete + Craving >= 400.

### Layer 3 — ANCHORING, "The Ledger" — Piper
- Lessons 5-6 (days 41-60). Vehicle: protocol. The
  yes-response trained as reflex; pleasure bound to
  answering the cue (L6 Pleasure Obeys).
- Presentation: kneeling keeper, open ledger; the
  day's ledger line is written in her hand-style font
  as the session completes.
- Activity: THE MANTRA — speech-to-text checks (local
  Whisper, the estate's QC lane in-game): the user
  speaks the layer's mantra lines aloud; the game
  verifies wording, not identity. Failed checks are
  practice, never punishment-lockouts: three attempts,
  then the line is spoken by the keeper and the day
  proceeds (the drill records the assist).
- Exit: L6 deep session complete + Yes >= 600 +
  at least one unassisted mantra pass.

### Layer 4 — CONDITIONING, "The Machine" — Sloane
- Lesson 7 (days 61-70). Vehicle: stimulus pairing.
  Timed sensation prompts (screen pulse, audio ticks,
  and — where the user has connected one — the
  Lovense lane behind its standing safety gate, off
  by default and never required) pair with the
  identity suggestions of L7 Who You Are Becoming.
- Presentation: candlelit staging, collar and mala;
  device readouts tick in time with the pairing
  schedule.
- Activity: THE APPRAISAL — a slow inspection routine:
  the keeper appraises the day's ledger and meters;
  the user holds stillness through it (her appraisal
  kink as attention training).
- Exit: L7 deep session complete + Pairing >= 700.

### Layer 5 — EMBODIMENT, "The Ritual" — Alma
- Lessons 8-9 (days 71-90). Vehicle: routine. The
  conditioning leaves the session chair and becomes
  the day itself: service rituals on a schedule,
  fullness and release as the rhythm (L8 Ache and
  Wait), then every anchor firing inside ordinary
  tasks (L9 Everything at Once).
- Presentation: the maid-service staging — her
  animations run the room's routine around the
  session; the sorting & cleaning minigame (the
  original's activity, estate-original presentation)
  IS this layer's activity: THE SERVICE — sort and
  set the room before the session; the room's state
  persists across days as the visible record of the
  ritual's upkeep.
- Exit: L9 deep session complete + Ritual >= 850.

### Layer 6 — TOTALITY, "The Spiral" — Prima
- Lesson 10 (days 91-100). Vehicle: total immersion —
  her corruption/TPE register at full strength; the
  60-minute Spiral with every installed anchor live.
- Presentation: her stage (the PL04 magenta-curtain
  spotlight staging); all five prior keepers' frames
  appear at the stage's edge in the finale sequence —
  the whole protocol present at the end of it.
- Activity: none. The last ten days are sessions and
  anchors only; the protocol no longer needs drills.
- Completion: day 100 installs the loop-back cue
  "begin again". The ledger closes with the line the
  series wrote for it, and the counter offers the
  only ending the design has: begin again at day 1 —
  willingly, which was the title all along.

## 6. Animation inventory and restaging

Per keeper, existing assets (verified on disk
2026-11-09, ~/workspace/torogao-models-20261104/):
- Build-3 rig (GLB + .blend where present), 54 bones,
  Basis + 36 facial morph targets, expression and
  param maps.
- anim-arc/: her solo arc show (BUILD -> BUILD II ->
  PEAK -> AFTERGLOW; <girl>-arc.mp4, frame strips,
  channel/skeleton JSON, deterministic replay proofs).
- anim-neutral/: her neutral show (<girl>-neutral.mp4)
  — the idle/presence loop class.
- 2D solo style sheets (cast-solo-sheets-20261104).
- Dream-style stills (asset factory, ntranch lane):
  per-girl best set of record (Prima v2, Alma v2,
  Piper v2, Sloane v6/v4, Yui v3; Lux recorded B is
  KEEPER, Owner 2026-11-09).
- Wan image-to-video clips: route in validation
  (Prima pilot running on primo at design time; LTX
  measured reference-class only). The design does not
  block on clips: session presentation launches from
  rig renders + stills; admitted clips slot into the
  same presentation slots when they land.

Restaging method (deterministic, per D-197 — no
neural runtime ships in the game): each keeper's
in-game loops are rendered from her rig under her
layer's staging (frame, lighting grade amber-to-
violet across the ladder per D-1091, overlay set per
section 5) and shipped as frozen video/sprite data.
The arc show is re-cut per layer: her layer uses the
arc segment matching its register (e.g. Lux presents
BUILD/BUILD II — the performance; Prima presents
PEAK/AFTERGLOW — the savoring).

## 7. Audio

- Sessions: the v2 lesson masters (production of
  record; per-lesson QC gates as they clear).
  Constants (D-1092): Kokoro af_heart, speed 0.73,
  48 kHz, -16 LUFS / TP -1.2, 200 Hz anchor ear,
  SSB-AM 17 kHz carrier, subliminal depth ladder
  -30 dB (L1) to -18 dB (L10), disclosed suggestion
  line in all ten.
- Editions: Full = deep sessions; Lite = practice
  days; Maintenance (native from L5) = later-block
  practice rotation.
- SFX library (screened; terms captured at ingest):
  Kuchiko BDSM Gear & Bodily Fluids 01 — CLEAR (any
  use incl. commercial; no resale as-is).
  MonsterBox Free Hentai/SFM — CLEAR with attribution
  (author page comment grade; credit in-game).
  MonsterBox Lip Service — FLAGGED/unresolved; the
  design does not depend on it. Owner purchase call
  ($2.50) would settle terms at ingest.
  Apex WET SEX — FLAGGED, zero license text; NOT in
  the design. Do not buy on copy alone.
- The keepers do not voice the lessons (the locked
  series voice does). Keeper text lines are
  estate-original English (D-472), written per layer.

## 8. Systems shape (for the S4 gate)

- One systems crate (Torogao worktree, coin/modapp
  pattern) + one RON data file carrying: the 100-day
  table (block/lesson/day-type), the layer table
  (keeper, vehicle, exit thresholds above), the cue
  table, the pairing schedule for Layer 4, the mantra
  lines for Layer 3.
- Loader refuses UNKNOWNs in every table.
- STT: local Whisper, offline only; audio of the
  user's voice is processed on-device and never
  stored (only pass/fail + attempt counts persist).
- Save: section 4's vocabulary, versioned; the S7
  determinism gate replays a scripted 100-day run on
  Linux and Android and compares state hashes.
- Tests at S4: half validation / half adversarial
  ( Tiger Style house split): malformed data files,
  out-of-vocabulary saves, skipped-day sequences,
  mantra-failure paths, surface-control interruption
  mid-session.

## 9. Open Owner gates

1. S4 commission (systems crate build) after this doc.
2. Lip Service $2.50 purchase (settles its terms; the
   design does not need it).
3. Key visual / cover art pass for the game page
   (the title's image: a fall, asked for).
4. itch page creation — waits on the project-limit
   raise; nothing in this doc blocks on it.
