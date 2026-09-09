use af_alarm::af::{NaiveDetector, CusumDetector};

// ============ Naive Detector Tests ============

#[test]
fn naive_no_alarm_below_threshold() {
    let mut det = NaiveDetector::new(0.5);
    det.update(0.1, 100);
    det.update(0.2, 200);
    det.update(0.49, 300);
    assert_eq!(det.alarms.len(), 0);
}

#[test]
fn naive_alarm_above_threshold() {
    let mut det = NaiveDetector::new(0.5);
    det.update(0.6, 100);
    assert_eq!(det.alarms.len(), 1);
    assert_eq!(det.alarms[0], 100);
}

#[test]
fn naive_rising_edge_only() {
    // Should only count rising edge, not every beat above threshold
    let mut det = NaiveDetector::new(0.5);
    det.update(0.6, 100);  // alarm
    det.update(0.7, 200);  // still above, no new alarm
    det.update(0.8, 300);  // still above, no new alarm
    assert_eq!(det.alarms.len(), 1);
}

#[test]
fn naive_rearm_after_drop() {
    let mut det = NaiveDetector::new(0.5);
    det.update(0.6, 100);  // alarm 1
    det.update(0.3, 200);  // drop below - rearm
    det.update(0.7, 300);  // alarm 2
    assert_eq!(det.alarms.len(), 2);
    assert_eq!(det.alarms[0], 100);
    assert_eq!(det.alarms[1], 300);
}

#[test]
fn naive_multiple_episodes() {
    let mut det = NaiveDetector::new(0.5);
    // Episode 1
    det.update(0.6, 100);
    det.update(0.7, 200);
    // Gap
    det.update(0.2, 300);
    det.update(0.1, 400);
    // Episode 2
    det.update(0.8, 500);
    det.update(0.9, 600);
    // Gap
    det.update(0.3, 700);
    // Episode 3
    det.update(0.55, 800);

    assert_eq!(det.alarms.len(), 3);
    assert_eq!(det.alarms, vec![100, 500, 800]);
}

// ============ CUSUM Detector Tests ============

#[test]
fn cusum_accumulation() {
    let mut det = CusumDetector::new(0.1, 1.0);  // k=0.1, h=1.0
    // S_t = max(0, S_{t-1} + z_t - k)
    det.update(0.3, 100);  // S = 0 + 0.3 - 0.1 = 0.2
    assert!((det.s_t - 0.2).abs() < 0.001);

    det.update(0.5, 200);  // S = 0.2 + 0.5 - 0.1 = 0.6
    assert!((det.s_t - 0.6).abs() < 0.001);

    det.update(0.6, 300);  // S = 0.6 + 0.6 - 0.1 = 1.1 > h, alarm!
    assert!((det.s_t - 1.1).abs() < 0.001);
    assert_eq!(det.alarms.len(), 1);
}

#[test]
fn cusum_reset_to_zero() {
    let mut det = CusumDetector::new(0.5, 1.0);  // k=0.5, h=1.0
    // Small values that subtract more than they add
    det.update(0.1, 100);  // S = 0 + 0.1 - 0.5 = -0.4 -> max(0, -0.4) = 0
    assert_eq!(det.s_t, 0.0);

    det.update(0.2, 200);  // S = 0 + 0.2 - 0.5 = -0.3 -> 0
    assert_eq!(det.s_t, 0.0);
}

#[test]
fn cusum_alarm_at_threshold() {
    let mut det = CusumDetector::new(0.0, 1.0);  // k=0, h=1.0
    det.update(0.5, 100);  // S = 0.5
    assert_eq!(det.alarms.len(), 0);

    det.update(0.5, 200);  // S = 1.0, not > h
    assert_eq!(det.alarms.len(), 0);

    det.update(0.1, 300);  // S = 1.1 > h, alarm!
    assert_eq!(det.alarms.len(), 1);
}

#[test]
fn cusum_rising_edge_only() {
    let mut det = CusumDetector::new(0.0, 0.5);  // k=0, h=0.5
    det.update(0.6, 100);  // S = 0.6 > h, alarm
    det.update(0.6, 200);  // S = 1.2 > h, but already in alarm
    det.update(0.6, 300);  // S = 1.8 > h, still in alarm

    assert_eq!(det.alarms.len(), 1);
    assert_eq!(det.alarms[0], 100);
}

#[test]
fn cusum_rearm_when_drops_below_h() {
    let mut det = CusumDetector::new(0.5, 1.0);  // k=0.5, h=1.0

    // Build up to alarm
    det.update(1.0, 100);  // S = 0.5
    det.update(1.0, 200);  // S = 1.0
    det.update(0.6, 300);  // S = 1.1 > h, alarm!
    assert_eq!(det.alarms.len(), 1);

    // Stay above h
    det.update(0.6, 400);  // S = 1.2
    assert_eq!(det.alarms.len(), 1);

    // Drop S below h by having low z_t values
    det.update(0.0, 500);  // S = 1.2 + 0 - 0.5 = 0.7 < h, rearm
    assert!(!det.in_alarm);

    // New alarm
    det.update(1.0, 600);  // S = 0.7 + 1.0 - 0.5 = 1.2 > h, alarm!
    assert_eq!(det.alarms.len(), 2);
    assert_eq!(det.alarms[1], 600);
}

// ============ AF Onset/Delay Tests ============

#[test]
fn cusum_delay_calculation() {
    // Simulate AF onset at sample 1000
    // Detector should alarm some time after
    let mut det = CusumDetector::new(0.1, 0.5);

    // Normal rhythm (low z_t)
    for i in 0..10 {
        det.update(0.05, i * 100);  // samples 0, 100, 200, ...
    }
    assert_eq!(det.alarms.len(), 0);

    // AF starts at sample 1000 (high z_t)
    det.update(0.3, 1000);  // S = 0 + 0.3 - 0.1 = 0.2
    det.update(0.3, 1100);  // S = 0.2 + 0.3 - 0.1 = 0.4
    det.update(0.3, 1200);  // S = 0.4 + 0.3 - 0.1 = 0.6 > h, alarm!

    assert_eq!(det.alarms.len(), 1);
    assert_eq!(det.alarms[0], 1200);
    // Delay = 1200 - 1000 = 200 samples
}
