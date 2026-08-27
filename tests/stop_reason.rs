use rusty_rival::types::{set_stop_reason, StopReason};
use std::sync::atomic::{AtomicU8, Ordering};

#[test]
fn first_stop_reason_wins() {
    let reason = AtomicU8::new(StopReason::None as u8);
    set_stop_reason(&reason, StopReason::MaxSoft);
    set_stop_reason(&reason, StopReason::External);
    assert_eq!(reason.load(Ordering::Relaxed), StopReason::MaxSoft as u8);
}

#[test]
fn external_observation_cannot_replace_a_local_cause() {
    let reason = AtomicU8::new(StopReason::None as u8);
    set_stop_reason(&reason, StopReason::Predictor);
    set_stop_reason(&reason, StopReason::External);
    assert_eq!(reason.load(Ordering::Relaxed), StopReason::Predictor as u8);
}
