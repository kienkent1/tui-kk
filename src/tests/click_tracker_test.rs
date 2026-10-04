use std::time::{Duration, Instant};

use crate::tuikk_core::click_tracker::{ClickTracker, Clicks, MAX_COUNT};

fn ms(n: u64) -> Duration {
    Duration::from_millis(n)
}

#[test]
fn counts_then_restarts() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    assert_eq!(t.listen_at(t0, 0, 1), Clicks::Click);
    assert_eq!(t.listen_at(t0 + ms(100), 0, 1), Clicks::DoubleClick);
    assert_eq!(t.listen_at(t0 + ms(200), 0, 1), Clicks::TripleClick);
    assert_eq!(t.listen_at(t0 + ms(300), 0, 1), Clicks::FourClicks);
    assert_eq!(t.count(), 4);

    // the 5th click starts a new sequence (MAX_COUNT = 4)
    assert_eq!(t.listen_at(t0 + ms(400), 0, 1), Clicks::Click);
    assert_eq!(t.count(), 1);
}

#[test]
fn different_target_breaks_chain() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    assert_eq!(t.listen_at(t0, 0, 1), Clicks::Click);
    // different row
    assert_eq!(t.listen_at(t0 + ms(100), 0, 2), Clicks::Click);
    // different col, same row
    assert_eq!(t.listen_at(t0 + ms(200), 5, 2), Clicks::Click);
    assert_eq!(t.count(), 1);
}

#[test]
fn noise_is_ignored_and_keeps_state() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    assert_eq!(t.listen_at(t0, 0, 1), Clicks::Click);
    // < MIN_CLICK_GAP (30ms): ignored, state unchanged
    assert_eq!(t.listen_at(t0 + ms(10), 0, 1), Clicks::Nothing);
    assert_eq!(t.count(), 1);
    assert_eq!(t.click(), Clicks::Click);

    // the noise did not refresh `last`, so this is measured from t0
    assert_eq!(t.listen_at(t0 + ms(100), 0, 1), Clicks::DoubleClick);
}

#[test]
fn gap_too_long_restarts() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    assert_eq!(t.listen_at(t0, 0, 1), Clicks::Click);
    // > MULTI_CLICK_GAP (400ms)
    assert_eq!(t.listen_at(t0 + ms(500), 0, 1), Clicks::Click);
    assert_eq!(t.count(), 1);
}

#[test]
fn total_duration_limit_restarts() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    // each gap is 350ms (< 400ms), but the total exceeds 900ms
    assert_eq!(t.listen_at(t0, 0, 1), Clicks::Click);
    assert_eq!(t.listen_at(t0 + ms(350), 0, 1), Clicks::DoubleClick);
    assert_eq!(t.listen_at(t0 + ms(700), 0, 1), Clicks::TripleClick);
    // 1050ms from the first click > MULTI_CLICK_TOTAL
    assert_eq!(t.listen_at(t0 + ms(1050), 0, 1), Clicks::Click);
}

#[test]
fn reset_clears_everything() {
    let mut t = ClickTracker::new();
    let t0 = Instant::now();

    t.listen_at(t0, 0, 1);
    t.listen_at(t0 + ms(100), 0, 1);
    assert_eq!(t.count(), 2);

    t.reset();
    assert_eq!(t.count(), 0);
    assert_eq!(t.click(), Clicks::Nothing);

    // after reset, the next click starts a fresh chain
    assert_eq!(t.listen_at(t0 + ms(150), 0, 1), Clicks::Click);
}

#[test]
fn max_count_matches_enum() {
    assert_eq!(Clicks::from_count(MAX_COUNT), Clicks::FourClicks);
    assert_eq!(Clicks::from_count(MAX_COUNT + 1), Clicks::Nothing);
}
