//! fall-smoke: a deterministic ten-day compressed run
//! of the Where She Asks to Fall protocol on the design
//! tables — Permission, then the first block's four
//! beats a day against injected calendar dates. Prints
//! the outcome and the run's hashes; exit 0 on a
//! completed run, 1 on refusal. This is the skeleton's
//! Linux-native proof artifact, not a game. The printed
//! state hash is the S7 cross-platform comparison value.

use torogao_fall::run_design_fixture;

fn main() {
    let code = run();
    std::process::exit(code);
}

fn run() -> i32 {
    match run_design_fixture() {
        Ok(run) => {
            println!(
                "fall-smoke: success=true days={} steps={} anchors={:010b} depth={} state_hash={:016x} trajectory_hash={:016x}",
                run.days_completed,
                run.steps_executed,
                run.anchors_installed,
                run.depth_meter,
                run.final_state_hash,
                run.trajectory_hash
            );
            0
        }
        Err(e) => {
            eprintln!("fall-smoke: {e}");
            1
        }
    }
}
