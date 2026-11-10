//! End-to-end tests for the torogao-fall protocol core.
//!
//! The house split (design section 8): half validation,
//! half adversarial. Validation tests are prefixed
//! `validation_`, adversarial tests `adversarial_`, so
//! the split is countable from the test list. The
//! scripted trainee plays one protocol day per calendar
//! date, starting 2027-03-01 — every date is injected,
//! the core never reads a clock.

use torogao_fall::{
    CalendarDate, FallError, MantraOutcome, MeterKind, PROTOCOL_RON, Protocol, SessionEffect,
    SessionKind, Settings, State, design_protocol, hash_state, run_design_fixture,
};

fn protocol() -> Protocol {
    design_protocol().expect("the design protocol loads")
}

/// 2027-03-01 plus `days` calendar days.
fn date_plus(days: u32) -> CalendarDate {
    let mut date = CalendarDate::new(2027, 3, 1).expect("the base date is valid");
    for _ in 0..days {
        date = date.next_day().expect("the scripted range stays in range");
    }
    date
}

/// Play the current day's four beats in order.
fn run_day(state: &mut State, protocol: &Protocol, on: CalendarDate) {
    let cue = if state.anchors() == 0 { None } else { Some(1) };
    state.record_anchor_check(cue, on).expect("anchor beat");
    if state.layer() != 6 {
        state.record_activity(protocol).expect("activity beat");
    }
    let row = protocol
        .day_row(state.day())
        .expect("the current day has a row");
    state
        .complete_session(protocol, row.lesson, row.session, on)
        .expect("session beat");
    state.record_ledger().expect("ledger beat");
}

/// Play from the current day through the end of protocol
/// day `through`, one calendar date per day (protocol
/// day N plays on 2027-03-01 + (N - 1) days).
fn advance_through(state: &mut State, protocol: &Protocol, through: u16) {
    while state.day() <= through && !state.is_finished() {
        let on = date_plus(u32::from(state.day() - 1));
        run_day(state, protocol, on);
    }
}

fn granted() -> (State, Protocol) {
    let protocol = protocol();
    let mut state = State::new();
    state.grant_permission(date_plus(0)).expect("permission");
    (state, protocol)
}

/// Replace the number after `key` in serialized RON.
fn replace_number_after(text: &str, key: &str, replacement: &str) -> String {
    let start = text.find(key).expect("the key is present") + key.len();
    let tail = &text[start..];
    let digits = tail.chars().take_while(char::is_ascii_digit).count();
    format!("{}{}{}", &text[..start], replacement, &tail[digits..])
}

// ---------------------------------------------------------------- validation

#[test]
fn validation_full_run_reaches_day_100_and_loops_back() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 41);
    state
        .record_mantra_attempt(
            &protocol,
            "mantra-1",
            "I answer yes, and my answer is mine.",
        )
        .expect("a mantra attempt in layer 3");
    advance_through(&mut state, &protocol, 100);
    assert!(state.is_finished());
    assert_eq!(state.day(), 100);
    assert_eq!(state.anchors(), 0b11_1111_1111);
    for lesson in 1..=10u8 {
        let counts = state.counts(lesson).expect("the lesson exists");
        assert_eq!(counts.deep, 1, "lesson {lesson} deep count");
        assert_eq!(counts.practice, 9, "lesson {lesson} practice count");
    }
    assert_eq!(state.meter(MeterKind::Depth), 288);
    assert_eq!(state.meter(MeterKind::Yes), 667);
    state.begin_again(date_plus(100)).expect("begin again");
    assert!(state.is_looped());
    assert_eq!(state.day(), 1);
    assert_eq!(state.anchors(), 0);
    assert_eq!(state.meter(MeterKind::Depth), 0);
    run_day(&mut state, &protocol, date_plus(100));
    assert_eq!(state.day(), 2);
}

#[test]
fn validation_pacing_unlocks_on_the_next_date() {
    let (mut state, protocol) = granted();
    run_day(&mut state, &protocol, date_plus(0));
    let err = state
        .record_anchor_check(None, date_plus(0))
        .expect_err("day 2 is locked on day 1's date");
    assert!(matches!(err, FallError::DayLocked { day: 2, .. }));
    state
        .record_anchor_check(Some(1), date_plus(1))
        .expect("day 2 opens on the next date");
}

#[test]
fn validation_practice_replay_leaves_day_and_clock_untouched() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 11);
    assert_eq!(state.counts(1).expect("lesson 1").practice, 9);
    let effect = state
        .complete_session(&protocol, 1, SessionKind::Practice, date_plus(11))
        .expect("a replay of an installed lesson");
    assert_eq!(effect, SessionEffect::Replay);
    assert_eq!(state.day(), 12);
    assert_eq!(state.unlock_date(), Some(date_plus(11)));
    assert!(!state.anchor_done());
    assert_eq!(state.counts(1).expect("lesson 1").practice, 10);
}

#[test]
fn validation_mantra_unassisted_pass_counts_toward_piper_exit() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 40);
    let outcome = state
        .record_mantra_attempt(
            &protocol,
            "mantra-2",
            "the ledger is open, and i am honest in it",
        )
        .expect("an attempt on day 41");
    assert_eq!(outcome, MantraOutcome::Pass);
    assert!(state.mantra_passed_unassisted());
    advance_through(&mut state, &protocol, 60);
    assert!(state.layer_exit_met(&protocol, 3).expect("exit query"));
    assert!(state.meter(MeterKind::Yes) >= 600);
}

#[test]
fn validation_layer_exits_open_in_order() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 10);
    assert!(!state.layer_exit_met(&protocol, 1).expect("exit query"));
    advance_through(&mut state, &protocol, 20);
    assert!(state.layer_exit_met(&protocol, 1).expect("exit query"));
    advance_through(&mut state, &protocol, 40);
    assert!(state.layer_exit_met(&protocol, 2).expect("exit query"));
    advance_through(&mut state, &protocol, 41);
    state
        .record_mantra_attempt(
            &protocol,
            "mantra-1",
            "I answer yes, and my answer is mine.",
        )
        .expect("a mantra attempt in layer 3");
    advance_through(&mut state, &protocol, 60);
    assert!(state.layer_exit_met(&protocol, 3).expect("exit query"));
    advance_through(&mut state, &protocol, 70);
    assert!(state.layer_exit_met(&protocol, 4).expect("exit query"));
    advance_through(&mut state, &protocol, 90);
    assert!(state.layer_exit_met(&protocol, 5).expect("exit query"));
    advance_through(&mut state, &protocol, 100);
    assert!(state.layer_exit_met(&protocol, 6).expect("exit query"));
}

#[test]
fn validation_mantra_third_failure_records_an_assist() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 40);
    let wrong = "something else entirely";
    assert_eq!(
        state
            .record_mantra_attempt(&protocol, "mantra-3", wrong)
            .expect("attempt 1"),
        MantraOutcome::Fail
    );
    assert_eq!(
        state
            .record_mantra_attempt(&protocol, "mantra-3", wrong)
            .expect("attempt 2"),
        MantraOutcome::Fail
    );
    assert_eq!(
        state
            .record_mantra_attempt(&protocol, "mantra-3", wrong)
            .expect("attempt 3"),
        MantraOutcome::Assisted
    );
    let history = state.mantra_history();
    assert_eq!(history[history.len() - 1].outcome, MantraOutcome::Assisted);
    // The day proceeds: layer 3's beats still run.
    run_day(&mut state, &protocol, date_plus(40));
    assert_eq!(state.day(), 42);
}

#[test]
fn validation_save_round_trips_a_mid_run_state() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 44);
    state
        .record_anchor_check(Some(1), date_plus(44))
        .expect("anchor beat");
    state.record_activity(&protocol).expect("activity beat");
    state
        .record_mantra_attempt(&protocol, "mantra-4", "not the line")
        .expect("a failed attempt is recorded");
    let text = state.to_ron().expect("serialize");
    let loaded = State::from_ron(&text).expect("the save loads back");
    assert_eq!(loaded, state);
    assert_eq!(hash_state(&loaded), hash_state(&state));
}

#[test]
fn validation_fixture_hash_is_deterministic() {
    let a = run_design_fixture().expect("the first fixture run");
    let b = run_design_fixture().expect("the second fixture run");
    assert_eq!(a, b);
    assert_eq!(a.days_completed, 10);
    assert_eq!(a.anchors_installed, 0b1);
}

#[test]
fn validation_design_data_matches_the_doc_shape() {
    let protocol = protocol();
    assert_eq!(protocol.days().len(), 100);
    assert_eq!(
        protocol.day_row(1).expect("day 1").session,
        SessionKind::Deep
    );
    assert_eq!(protocol.day_row(11).expect("day 11").lesson, 2);
    assert_eq!(
        protocol.day_row(91).expect("day 91").session,
        SessionKind::Deep
    );
    let last = protocol.day_row(100).expect("day 100");
    assert_eq!(last.session, SessionKind::Practice);
    assert_eq!(last.lesson, 10);
    assert_eq!(protocol.layer_row(1).expect("layer 1").keeper, "Lux");
    assert_eq!(protocol.layer_row(4).expect("layer 4").keeper, "Sloane");
    assert!(protocol.layer_row(3).expect("layer 3").exit_mantra_required);
    assert_eq!(protocol.cue(2).expect("cue 2").cue, "deep and calm");
    assert_eq!(protocol.cue(8).expect("cue 8").cue, "now, for me");
    assert_eq!(protocol.cue(10).expect("cue 10").cue, "begin again");
    assert!(protocol.mantra_lines().len() >= 4);
    assert_eq!(protocol.pairing_schedule().len(), 6);
    assert_eq!(protocol.pairing_schedule()[5].intensity_percent, 100);
}

#[test]
fn validation_maintenance_replay_counts_as_practice() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 41);
    assert_eq!(state.counts(5).expect("lesson 5").practice, 0);
    let effect = state
        .complete_session(&protocol, 5, SessionKind::Maintenance, date_plus(41))
        .expect("a maintenance replay of an installed lesson");
    assert_eq!(effect, SessionEffect::Replay);
    assert_eq!(state.counts(5).expect("lesson 5").practice, 1);
    assert_eq!(state.day(), 42);
}

#[test]
fn validation_settings_update_validates_and_sticks() {
    let (mut state, _protocol) = granted();
    let mut settings: Settings = state.settings().clone();
    settings.volume_percent = 100;
    settings.pendulum = false;
    state.update_settings(settings).expect("valid settings");
    assert_eq!(state.settings().volume_percent, 100);
    assert!(!state.settings().pendulum);
}

// -------------------------------------------------------------- adversarial

#[test]
fn adversarial_data_with_an_unknown_field_is_refused() {
    let bad = PROTOCOL_RON.replacen(
        "schema: \"torogao-fall-protocol/1\"",
        "schema: \"torogao-fall-protocol/1\", unknown_field: 7",
        1,
    );
    assert!(Protocol::load(&bad).is_err());
}

#[test]
fn adversarial_data_with_99_days_is_refused() {
    let bad = PROTOCOL_RON
        .lines()
        .filter(|line| !line.contains("(day: 100,"))
        .collect::<Vec<_>>()
        .join("\n");
    let err = Protocol::load(&bad).expect_err("99 days is not the protocol");
    assert!(matches!(err, FallError::Invalid { field: "days", .. }));
}

#[test]
fn adversarial_save_with_an_unknown_field_is_refused() {
    let (state, _protocol) = granted();
    let good = state.to_ron().expect("serialize");
    let bad = format!("{}, unknown_field: 1)", &good[..good.len() - 1]);
    assert!(State::from_ron(&bad).is_err());
}

#[test]
fn adversarial_save_with_a_future_version_is_refused() {
    let (state, _protocol) = granted();
    let good = state.to_ron().expect("serialize");
    let bad = replace_number_after(&good, "format_version:", "2");
    let err = State::from_ron(&bad).expect_err("a future version is refused");
    assert!(matches!(
        err,
        FallError::SaveInvalid {
            field: "format_version",
            ..
        }
    ));
}

#[test]
fn adversarial_beats_before_the_unlock_date_are_refused_and_state_unchanged() {
    let (mut state, protocol) = granted();
    run_day(&mut state, &protocol, date_plus(0));
    let before = state.to_ron().expect("serialize");
    let err = state
        .record_anchor_check(None, date_plus(0))
        .expect_err("day 2's beats are locked on day 1's date");
    assert!(matches!(err, FallError::DayLocked { day: 2, .. }));
    assert_eq!(state.to_ron().expect("serialize"), before);
    state
        .record_anchor_check(Some(1), date_plus(1))
        .expect("anchor on the unlock date");
    state.record_activity(&protocol).expect("activity");
    let before = state.to_ron().expect("serialize");
    let err = state
        .complete_session(&protocol, 1, SessionKind::Practice, date_plus(0))
        .expect_err("a backdated session is locked");
    assert!(matches!(err, FallError::DayLocked { day: 2, .. }));
    assert_eq!(state.to_ron().expect("serialize"), before);
}

#[test]
fn adversarial_day_skip_deep_is_refused() {
    let (mut state, protocol) = granted();
    let before = state.to_ron().expect("serialize");
    let err = state
        .complete_session(&protocol, 2, SessionKind::Deep, date_plus(0))
        .expect_err("lesson 2's deep session is not today");
    assert!(matches!(err, FallError::DeepNotScheduled { lesson: 2 }));
    let err = state
        .complete_session(&protocol, 1, SessionKind::Practice, date_plus(0))
        .expect_err("day 1 schedules the deep session, not practice");
    assert!(matches!(err, FallError::AnchorNotInstalled { lesson: 1 }));
    assert_eq!(state.to_ron().expect("serialize"), before);
}

#[test]
fn adversarial_mantra_history_never_exceeds_eight() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 40);
    for attempt in 0..20 {
        let line = if attempt % 2 == 0 {
            "mantra-1"
        } else {
            "mantra-2"
        };
        state
            .record_mantra_attempt(&protocol, line, "not the line")
            .expect("attempts record even when they fail");
    }
    assert_eq!(state.mantra_history().len(), 8);
}

#[test]
fn adversarial_meters_saturate_at_the_cap() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 11);
    for _ in 0..300 {
        state
            .complete_session(&protocol, 1, SessionKind::Practice, date_plus(11))
            .expect("replays keep recording");
    }
    assert_eq!(state.meter(MeterKind::Depth), 1000);
    assert_eq!(state.counts(1).expect("lesson 1").practice, 309);
    assert_eq!(state.day(), 12);
}

#[test]
fn adversarial_invalid_calendar_dates_are_refused() {
    for (year, month, day) in [
        (2027, 2, 30),
        (2027, 2, 29),
        (2027, 4, 31),
        (2027, 13, 1),
        (2027, 0, 10),
        (2027, 1, 0),
        (0, 1, 1),
        (10000, 1, 1),
    ] {
        assert!(
            CalendarDate::new(year, month, day).is_err(),
            "{year}-{month}-{day} must be refused"
        );
    }
}

#[test]
fn adversarial_session_completion_is_atomic() {
    let (mut state, protocol) = granted();
    run_day(&mut state, &protocol, date_plus(0));
    state
        .record_anchor_check(Some(1), date_plus(1))
        .expect("anchor beat");
    state.record_activity(&protocol).expect("activity beat");
    let before = state.to_ron().expect("serialize");
    // Passes the early checks (started, lesson known) and
    // fails a late one: lesson 1 has no maintenance
    // edition. Nothing may move.
    let err = state
        .complete_session(&protocol, 1, SessionKind::Maintenance, date_plus(1))
        .expect_err("lesson 1 has no maintenance edition");
    assert!(matches!(
        err,
        FallError::MaintenanceNotAvailable { lesson: 1 }
    ));
    let err = state
        .complete_session(&protocol, 5, SessionKind::Maintenance, date_plus(1))
        .expect_err("lesson 5's anchor is not installed yet");
    assert!(matches!(err, FallError::AnchorNotInstalled { lesson: 5 }));
    assert_eq!(state.to_ron().expect("serialize"), before);
    assert!(state.anchor_done() && state.activity_done());
}

#[test]
fn adversarial_beats_before_permission_are_refused() {
    let protocol = protocol();
    let mut state = State::new();
    let on = date_plus(0);
    assert!(matches!(
        state.record_anchor_check(None, on),
        Err(FallError::NotStarted)
    ));
    assert!(matches!(
        state.record_activity(&protocol),
        Err(FallError::NotStarted)
    ));
    assert!(matches!(
        state.complete_session(&protocol, 1, SessionKind::Deep, on),
        Err(FallError::NotStarted)
    ));
    assert!(matches!(state.record_ledger(), Err(FallError::NotStarted)));
    assert!(matches!(
        state.record_mantra_attempt(&protocol, "mantra-1", "x"),
        Err(FallError::NotStarted)
    ));
    state.grant_permission(on).expect("the first grant");
    assert!(matches!(
        state.grant_permission(on),
        Err(FallError::AlreadyStarted)
    ));
}

#[test]
fn adversarial_tampered_save_with_wrong_block_is_refused() {
    let (mut state, protocol) = granted();
    advance_through(&mut state, &protocol, 25);
    assert_eq!(state.block(), 3);
    let good = state.to_ron().expect("serialize");
    let bad = replace_number_after(&good, "block:", "9");
    let err = State::from_ron(&bad).expect_err("a stored block that disagrees is refused");
    assert!(matches!(err, FallError::SaveInvalid { .. }));
}
