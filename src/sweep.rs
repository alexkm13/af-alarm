use crate::rr::RRProcessor;
use crate::af::{NaiveDetector, CusumDetector};
use crate::eval::{AFEpisode, evaluate_detector, EvalResult};

#[derive(Debug, Clone)]
pub struct SweepPoint {
    pub detector: String,
    pub params: String,
    pub sensitivity: f64,      
    pub false_alarm_rate: f64, 
    pub mean_delay: f64,       
}

impl SweepPoint {
    
    pub fn dominates(&self, other: &SweepPoint) -> bool {
        self.sensitivity >= other.sensitivity
            && self.false_alarm_rate <= other.false_alarm_rate
            && self.mean_delay <= other.mean_delay
            && (self.sensitivity > other.sensitivity
                || self.false_alarm_rate < other.false_alarm_rate
                || self.mean_delay < other.mean_delay)
    }
}

fn run_naive_on_irregularity(
    irregularity: &[(f64, u32)],
    threshold: f64,
) -> Vec<u32> {
    let mut detector = NaiveDetector::new(threshold);
    for &(z_t, beat) in irregularity {
        detector.update(z_t, beat);
    }
    detector.alarms
}

fn run_cusum_on_irregularity(
    irregularity: &[(f64, u32)],
    k: f64,
    h: f64,
) -> Vec<u32> {
    let mut detector = CusumDetector::new(k, h);
    for &(z_t, beat) in irregularity {
        detector.update(z_t, beat);
    }
    detector.alarms
}

pub fn sweep_naive(
    irregularity: &[(f64, u32)],
    af_episodes: &[AFEpisode],
    record_duration: u32,
    thresholds: &[f64],
) -> Vec<SweepPoint> {
    let mut results = Vec::new();

    for &thresh in thresholds {
        let alarms = run_naive_on_irregularity(irregularity, thresh);
        let eval = evaluate_detector("naive", &alarms, af_episodes, record_duration);

        let sensitivity = if eval.total_af_episodes > 0 {
            eval.detected_episodes as f64 / eval.total_af_episodes as f64
        } else {
            1.0
        };

        results.push(SweepPoint {
            detector: "naive".to_string(),
            params: format!("thresh={:.2}", thresh),
            sensitivity,
            false_alarm_rate: eval.false_alarm_rate,
            mean_delay: if eval.mean_delay.is_nan() { f64::INFINITY } else { eval.mean_delay },
        });
    }

    results
}

pub fn sweep_cusum(
    irregularity: &[(f64, u32)],
    af_episodes: &[AFEpisode],
    record_duration: u32,
    k_values: &[f64],
    h_values: &[f64],
) -> Vec<SweepPoint> {
    let mut results = Vec::new();

    for &k in k_values {
        for &h in h_values {
            let alarms = run_cusum_on_irregularity(irregularity, k, h);
            let eval = evaluate_detector("cusum", &alarms, af_episodes, record_duration);

            let sensitivity = if eval.total_af_episodes > 0 {
                eval.detected_episodes as f64 / eval.total_af_episodes as f64
            } else {
                1.0
            };

            results.push(SweepPoint {
                detector: "cusum".to_string(),
                params: format!("k={:.2},h={:.1}", k, h),
                sensitivity,
                false_alarm_rate: eval.false_alarm_rate,
                mean_delay: if eval.mean_delay.is_nan() { f64::INFINITY } else { eval.mean_delay },
            });
        }
    }

    results
}

pub fn pareto_frontier(points: &[SweepPoint]) -> Vec<SweepPoint> {
    let mut frontier = Vec::new();

    for point in points {
        let dominated = points.iter().any(|other| other.dominates(point));
        if !dominated {
            frontier.push(point.clone());
        }
    }

    
    frontier.sort_by(|a, b| a.false_alarm_rate.partial_cmp(&b.false_alarm_rate).unwrap());
    frontier
}

pub fn match_far(
    naive_points: &[SweepPoint],
    cusum_points: &[SweepPoint],
    tolerance: f64,
) -> Vec<(SweepPoint, SweepPoint)> {
    let mut matches = Vec::new();

    for n in naive_points {
        for c in cusum_points {
            let far_diff = (n.false_alarm_rate - c.false_alarm_rate).abs();
            let max_far = n.false_alarm_rate.max(c.false_alarm_rate);
            if max_far > 0.0 && far_diff / max_far < tolerance {
                matches.push((n.clone(), c.clone()));
            } else if max_far == 0.0 && far_diff < 0.1 {
                matches.push((n.clone(), c.clone()));
            }
        }
    }

    matches
}

pub fn print_sweep_results(points: &[SweepPoint], title: &str) {
    println!("\n{}", title);
    println!("{:-<70}", "");
    println!("{:20} {:>12} {:>12} {:>12}", "Params", "Sens", "FAR/hr", "Delay(s)");
    println!("{:-<70}", "");

    for p in points {
        println!(
            "{:20} {:>11.1}% {:>12.1} {:>12.1}",
            p.params,
            p.sensitivity * 100.0,
            p.false_alarm_rate,
            p.mean_delay
        );
    }
}
