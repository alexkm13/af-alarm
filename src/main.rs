mod wfdb;
mod rr;
mod detector;
mod spsc;
mod af;
mod eval;
mod sweep;

use eval::{run_reference_path, evaluate_detector, AFEpisode};
use sweep::{sweep_naive, sweep_cusum, pareto_frontier, SweepPoint};

// All MIT-BIH records with AFIB annotations
const AFIB_RECORDS: &[&str] = &["201", "202", "203", "210", "217", "219", "221", "222"];

const SAMPLE_RATE: f64 = 360.0;

fn main() {
    println!("========== MULTI-RECORD AF ALARM EVALUATION ==========\n");

    // Aggregate data across all records
    let mut all_naive_results: Vec<SweepPoint> = Vec::new();
    let mut all_cusum_results: Vec<SweepPoint> = Vec::new();

    // Per-record stats
    let mut total_af_episodes = 0;
    let mut total_af_duration_s = 0.0;
    let mut total_non_af_hours = 0.0;

    // Collect all irregularity data and episodes for aggregate sweep
    let mut all_irregularity: Vec<(f64, u32)> = Vec::new();
    let mut all_episodes: Vec<AFEpisode> = Vec::new();
    let mut episode_record_offsets: Vec<(usize, u32)> = Vec::new(); // (record_idx, sample_offset)

    let mut record_sample_offset: u32 = 0;

    // Threshold and CUSUM grids
    let thresholds: Vec<f64> = (1..=19).map(|i| i as f64 * 0.05).collect();
    let k_values: Vec<f64> = vec![0.1, 0.2, 0.25, 0.30, 0.35, 0.40, 0.45, 0.50];
    let h_values: Vec<f64> = vec![0.3, 0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 5.0];

    println!("Processing {} records with AFIB annotations...\n", AFIB_RECORDS.len());

    for (rec_idx, &record) in AFIB_RECORDS.iter().enumerate() {
        let atr_path = format!("data/mit-bih-arrhythmia-database-1.0.0/{}.atr", record);

        let (ref_processor, af_episodes, record_end) = run_reference_path(&atr_path);

        if af_episodes.is_empty() {
            println!("Record {}: No AF episodes found, skipping", record);
            continue;
        }

        // Record stats
        let af_duration: f64 = af_episodes.iter()
            .map(|e| (e.end_sample - e.start_sample) as f64 / SAMPLE_RATE)
            .sum();
        let non_af_samples: u32 = record_end.saturating_sub(
            af_episodes.iter().map(|e| e.end_sample - e.start_sample).sum()
        );
        let non_af_hours = non_af_samples as f64 / SAMPLE_RATE / 3600.0;

        println!("Record {}: {} AF episodes ({:.0}s total), {:.2} non-AF hours",
            record, af_episodes.len(), af_duration, non_af_hours);

        total_af_episodes += af_episodes.len();
        total_af_duration_s += af_duration;
        total_non_af_hours += non_af_hours;

        // Add to aggregate data with offset
        for &(z_t, sample) in &ref_processor.irregularity {
            all_irregularity.push((z_t, sample + record_sample_offset));
        }
        for ep in &af_episodes {
            all_episodes.push(AFEpisode {
                start_sample: ep.start_sample + record_sample_offset,
                end_sample: ep.end_sample + record_sample_offset,
            });
            episode_record_offsets.push((rec_idx, record_sample_offset));
        }

        record_sample_offset += record_end + 10000; // Gap between records
    }

    let total_record_duration = record_sample_offset;

    println!("\n========== AGGREGATE STATISTICS ==========\n");
    println!("Total AF episodes: {}", total_af_episodes);
    println!("Total AF duration: {:.1} minutes", total_af_duration_s / 60.0);
    println!("Total non-AF time: {:.2} hours", total_non_af_hours);

    // Z_t distribution analysis
    let mut af_zt: Vec<f64> = Vec::new();
    let mut non_af_zt: Vec<f64> = Vec::new();

    for &(z_t, sample) in &all_irregularity {
        let in_af = all_episodes.iter().any(|e| sample >= e.start_sample && sample < e.end_sample);
        if in_af {
            af_zt.push(z_t);
        } else {
            non_af_zt.push(z_t);
        }
    }

    fn percentile(vals: &mut Vec<f64>, p: f64) -> f64 {
        if vals.is_empty() { return 0.0; }
        vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
        vals[(vals.len() as f64 * p) as usize]
    }

    let af_mean = af_zt.iter().sum::<f64>() / af_zt.len() as f64;
    let non_af_mean = non_af_zt.iter().sum::<f64>() / non_af_zt.len() as f64;

    println!("\nZ_t distributions:");
    println!("  AF periods ({} samples): mean={:.3}, median={:.3}, p90={:.3}",
        af_zt.len(), af_mean, percentile(&mut af_zt.clone(), 0.5), percentile(&mut af_zt.clone(), 0.9));
    println!("  Non-AF periods ({} samples): mean={:.3}, median={:.3}, p90={:.3}",
        non_af_zt.len(), non_af_mean, percentile(&mut non_af_zt.clone(), 0.5), percentile(&mut non_af_zt.clone(), 0.9));

    // Run parameter sweep on aggregate data
    println!("\n========== AGGREGATE PARAMETER SWEEP ==========\n");

    let naive_results = sweep_naive(&all_irregularity, &all_episodes, total_record_duration, &thresholds);
    let cusum_results = sweep_cusum(&all_irregularity, &all_episodes, total_record_duration, &k_values, &h_values);

    // Pareto frontiers
    let naive_pareto = pareto_frontier(&naive_results);
    let cusum_pareto = pareto_frontier(&cusum_results);

    println!("Naive Pareto Frontier ({} points):", naive_pareto.len());
    println!("{:-<80}", "");
    println!("{:20} {:>10} {:>12} {:>12} {:>12}", "Params", "Sens", "FAR/hr", "Mean Delay", "Episodes");
    println!("{:-<80}", "");
    for p in &naive_pareto {
        let detected = (p.sensitivity * total_af_episodes as f64).round() as usize;
        println!("{:20} {:>9.1}% {:>12.1} {:>11.1}s {:>8}/{}",
            p.params, p.sensitivity * 100.0, p.false_alarm_rate, p.mean_delay, detected, total_af_episodes);
    }

    println!("\nCUSUM Pareto Frontier ({} points):", cusum_pareto.len());
    println!("{:-<80}", "");
    println!("{:20} {:>10} {:>12} {:>12} {:>12}", "Params", "Sens", "FAR/hr", "Mean Delay", "Episodes");
    println!("{:-<80}", "");
    for p in &cusum_pareto {
        let detected = (p.sensitivity * total_af_episodes as f64).round() as usize;
        println!("{:20} {:>9.1}% {:>12.1} {:>11.1}s {:>8}/{}",
            p.params, p.sensitivity * 100.0, p.false_alarm_rate, p.mean_delay, detected, total_af_episodes);
    }

    // Matched FAR comparison
    println!("\n========== MATCHED FAR COMPARISONS ==========\n");

    // Find pairs within tolerance
    for tolerance in [0.3, 0.5] {
        println!("--- Within {:.0}% FAR tolerance ---", tolerance * 100.0);
        let mut found_any = false;

        for n in &naive_pareto {
            for c in &cusum_pareto {
                let far_diff = (n.false_alarm_rate - c.false_alarm_rate).abs();
                let max_far = n.false_alarm_rate.max(c.false_alarm_rate);
                let within_tol = if max_far > 0.0 {
                    far_diff / max_far < tolerance
                } else {
                    far_diff < 1.0
                };

                if within_tol && (n.sensitivity - c.sensitivity).abs() < 0.15 {
                    found_any = true;
                    println!("\nNaive {:15} vs CUSUM {:15}", n.params, c.params);
                    println!("  Sensitivity: {:>6.1}% vs {:>6.1}% (Δ={:+.1}%)",
                        n.sensitivity * 100.0, c.sensitivity * 100.0,
                        (c.sensitivity - n.sensitivity) * 100.0);
                    println!("  FAR/hr:      {:>6.1} vs {:>6.1} (ratio={:.2}x)",
                        n.false_alarm_rate, c.false_alarm_rate,
                        if n.false_alarm_rate > 0.0 { c.false_alarm_rate / n.false_alarm_rate } else { 0.0 });
                    println!("  Delay:       {:>5.1}s vs {:>5.1}s (Δ={:+.1}s)",
                        n.mean_delay, c.mean_delay, c.mean_delay - n.mean_delay);
                }
            }
        }
        if !found_any {
            println!("No matched pairs found.\n");
        }
    }

    // Sensitivity-stratified comparison
    println!("\n========== SENSITIVITY-STRATIFIED COMPARISON ==========\n");

    for (sens_label, sens_min, sens_max) in [
        ("≥90%", 0.90, 1.01),
        ("70-89%", 0.70, 0.90),
        ("50-69%", 0.50, 0.70),
        ("<50%", 0.0, 0.50),
    ] {
        let naive_in_range: Vec<&SweepPoint> = naive_pareto.iter()
            .filter(|p| p.sensitivity >= sens_min && p.sensitivity < sens_max)
            .collect();
        let cusum_in_range: Vec<&SweepPoint> = cusum_pareto.iter()
            .filter(|p| p.sensitivity >= sens_min && p.sensitivity < sens_max)
            .collect();

        if naive_in_range.is_empty() && cusum_in_range.is_empty() {
            continue;
        }

        println!("At {} sensitivity:", sens_label);
        if let Some(n) = naive_in_range.first() {
            println!("  Best Naive: {} (sens={:.1}%, FAR={:.1}, delay={:.1}s)",
                n.params, n.sensitivity * 100.0, n.false_alarm_rate, n.mean_delay);
        } else {
            println!("  Naive: No config in this range");
        }
        if let Some(c) = cusum_in_range.first() {
            println!("  Best CUSUM: {} (sens={:.1}%, FAR={:.1}, delay={:.1}s)",
                c.params, c.sensitivity * 100.0, c.false_alarm_rate, c.mean_delay);
        } else {
            println!("  CUSUM: No config in this range");
        }
        println!();
    }

    // Delay distribution analysis
    println!("========== DELAY DISTRIBUTION ==========\n");

    // Get best configs at ~80% sensitivity for delay analysis
    let target_sens = 0.80;
    let naive_near_target: Option<&SweepPoint> = naive_results.iter()
        .filter(|p| (p.sensitivity - target_sens).abs() < 0.15)
        .min_by(|a, b| a.false_alarm_rate.partial_cmp(&b.false_alarm_rate).unwrap());
    let cusum_near_target: Option<&SweepPoint> = cusum_results.iter()
        .filter(|p| (p.sensitivity - target_sens).abs() < 0.15)
        .min_by(|a, b| a.false_alarm_rate.partial_cmp(&b.false_alarm_rate).unwrap());

    if let Some(n) = naive_near_target {
        println!("Naive near {:.0}% sens: {} (actual={:.1}%)",
            target_sens * 100.0, n.params, n.sensitivity * 100.0);
        println!("  FAR={:.1}/hr, Mean delay={:.1}s", n.false_alarm_rate, n.mean_delay);
    }
    if let Some(c) = cusum_near_target {
        println!("CUSUM near {:.0}% sens: {} (actual={:.1}%)",
            target_sens * 100.0, c.params, c.sensitivity * 100.0);
        println!("  FAR={:.1}/hr, Mean delay={:.1}s", c.false_alarm_rate, c.mean_delay);
    }

    println!("\n========== KEY FINDINGS ==========\n");

    // Find max sensitivity for each detector
    let max_naive_sens = naive_results.iter().map(|p| p.sensitivity).fold(0.0, f64::max);
    let max_cusum_sens = cusum_results.iter().map(|p| p.sensitivity).fold(0.0, f64::max);

    println!("Maximum achievable sensitivity:");
    println!("  Naive:  {:.1}%", max_naive_sens * 100.0);
    println!("  CUSUM:  {:.1}%", max_cusum_sens * 100.0);

    // At max shared sensitivity
    let shared_max = max_naive_sens.min(max_cusum_sens);
    let naive_at_max: Option<&SweepPoint> = naive_pareto.iter()
        .filter(|p| p.sensitivity >= shared_max - 0.05)
        .min_by(|a, b| a.false_alarm_rate.partial_cmp(&b.false_alarm_rate).unwrap());
    let cusum_at_max: Option<&SweepPoint> = cusum_pareto.iter()
        .filter(|p| p.sensitivity >= shared_max - 0.05)
        .min_by(|a, b| a.false_alarm_rate.partial_cmp(&b.false_alarm_rate).unwrap());

    if let (Some(n), Some(c)) = (naive_at_max, cusum_at_max) {
        println!("\nAt {:.0}%+ sensitivity (best FAR):", (shared_max - 0.05) * 100.0);
        println!("  Naive:  {} → FAR={:.1}/hr, delay={:.1}s", n.params, n.false_alarm_rate, n.mean_delay);
        println!("  CUSUM:  {} → FAR={:.1}/hr, delay={:.1}s", c.params, c.false_alarm_rate, c.mean_delay);

        if n.false_alarm_rate > 0.0 {
            let far_ratio = c.false_alarm_rate / n.false_alarm_rate;
            let delay_diff = c.mean_delay - n.mean_delay;
            println!("\n  CUSUM vs Naive: FAR {:.2}x, delay {:+.1}s", far_ratio, delay_diff);
        }
    }

    println!("\n==========================================");
}
