# Where She Asks to Fall

## 18+ — Adults Only

This game contains adult content. It is intended only for players 18 and older.

### Content notes

- A 100-day hypnosis training protocol played in the first person — you are the trainee; there is no separate heroine
- The session audio carries a disclosed subliminal suggestion layer under the audible lesson. The suggestion is disclosed here and in the game itself, never hidden
- The six layers include being watched and recorded, rope bondage and edging, spoken mantra checks, stimulus pairing, ritual service, and total-immersion hypnosis
- The fiction is a brainwashing protocol; the product contract is consent: an 18+ gate and a Permission act come before day 1, a SURFACE control stops any session in one press and is never hidden, and every session ends on an aftercare screen

All characters depicted are adults.

## A 100-day hypnosis training protocol where you are the trainee — and every day only happens because you asked for it.

<p align="center"><a href="https://github.com/NTRanch/where-she-asks-to-fall">GitHub</a> · <a href="https://ntranch.itch.io">itch.io studio page</a></p>

## Built with

![Rust](https://img.shields.io/badge/Rust-2024_edition-000000?logo=rust&logoColor=white) ![Neovim](https://img.shields.io/badge/Neovim-nightly-57A143?logo=neovim&logoColor=white)

## What you do

Six keepers run the protocol, one layer at a time: Lux and the camera that induces you, Yui and the rope that deepens you, Piper and the ledger that anchors your yes, Sloane and the machine that pairs sensation to suggestion, Alma and the ritual that moves the training into your day, and Prima, whose Spiral closes the hundred days. Each layer keeps two of the Surrender Curriculum's ten lessons; each lesson anchors a ten-day block.

A day has four beats: the anchor check (a cue an earlier lesson installed, answered by tap or by voice), the layer's activity — hold the pose, endure the wait, speak the mantra, submit to the appraisal, set the room — then the day's session, then one line written to the protocol ledger. Day 1 of a block is the deep session; the nine days after it are practice.

The pacing is the training: one new protocol day unlocks per real calendar day. Practice days and finished content replay freely, but the counter only moves when the calendar does.

## Features

- 100 days in 10 blocks of 10, calendar-paced — one new day per real day, replays always open
- Six layers, six keepers, six methods: being watched, fractionation, protocol, stimulus pairing, ritual service, total immersion
- The Surrender Curriculum's ten lessons as the session audio — Full editions anchor each block, Lite and Maintenance editions carry the practice days
- Voice mantra checks in Piper's layer, verified on-device by local speech-to-text; your voice is processed on the device and never stored
- One save vocabulary carries the whole protocol: the day counter, a 10-bit mask of installed cues, six meters (Depth, Craving, Yes, Pairing, Ritual, Spiral), and the pacing date — the same inputs replay to the same state on Linux and Android
- Consent furniture is mechanics, not a menu: the Permission act creates the day counter, the SURFACE control is on screen at all times, and aftercare ends every session

## Media

No screenshots yet — the key visual pass is an open gate, and this section gains its media when the shell build does.

## Status

**In development — systems crate gated, no playable build yet.** The game's systems core, the `torogao-fall` crate in this repo, is built and gated: the full 100-day table, the four-beat day loop, the six layers' meters and exit conditions, calendar pacing, mantra checks, and the versioned save vocabulary as pure state transitions over one RON data file. 28 tests, 0 failures; the canonical fixture's state hash is `c61148c2fb666721`, identical across runs. The playable shell — the Torogao engine binding for native Linux and Android — is the next rung. Design and build records live in [`docs/`](docs/). The itch page waits on the itch project-limit raise; this GitHub repo is the game's home meanwhile. Free on release.

## Credits

A Nice Thoughts Ranch game.

Made with Rust, for the Torogao engine.

An estate-original, clean-room design: the premise was studied from a public page only — no outside assets, text, or code are used. Session audio is the Surrender Curriculum series; its suggestion layer is disclosed on this page and in the game's credits screen.
