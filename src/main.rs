mod wfdb;
mod rr;
mod detector;

use std::collections::HashMap;

fn main() {
    // Focus on record 108 FP investigation
    test_record("108");
}

fn test_record(record: &str) {
    let dat_path = format!("data/mit-bih-arrhythmia-database-1.0.0/{}.dat", record);
    let atr_path = format!("data/mit-bih-arrhythmia-database-1.0.0/{}.atr", record);

    // Get reference beats from .atr
    let (ref_beats, state_changes) = wfdb::parse_annotations(&atr_path);
    let ref_samples: Vec<u32> = ref_beats.iter().map(|b| b.sample).collect();

    // Run detector
    let detected = detector::detector(&dat_path);

    println!("Record {}", record);
    println!("Reference beats: {}", ref_samples.len());
    println!("Detected beats: {}", detected.len());

    // Match with ±54 sample tolerance (±150ms at 360Hz)
    let tolerance: u32 = 54;
    let mut true_pos = 0;
    let mut matched_ref: Vec<bool> = vec![false; ref_samples.len()];

    for det in &detected {
        for (i, ref_s) in ref_samples.iter().enumerate() {
            if !matched_ref[i] && (*det as i64 - *ref_s as i64).abs() <= tolerance as i64 {
                true_pos += 1;
                matched_ref[i] = true;
                break;
            }
        }
    }

    // Collect FPs: detected but not matched to any reference
    let mut matched_det: Vec<bool> = vec![false; detected.len()];
    for (j, det) in detected.iter().enumerate() {
        for ref_s in &ref_samples {
            if (*det as i64 - *ref_s as i64).abs() <= tolerance as i64 {
                matched_det[j] = true;
                break;
            }
        }
    }

    let false_positives: Vec<u32> = detected.iter().enumerate()
        .filter(|(i, _)| !matched_det[*i])
        .map(|(_, &d)| d)
        .collect();

    let false_negatives: Vec<u32> = ref_samples.iter().enumerate()
        .filter(|(i, _)| !matched_ref[*i])
        .map(|(_, &r)| r)
        .collect();

    let false_neg = ref_samples.len() - true_pos;
    let false_pos = detected.len() - true_pos;

    let sensitivity = 100.0 * true_pos as f64 / ref_samples.len() as f64;
    let ppv = 100.0 * true_pos as f64 / detected.len() as f64;

    // Summary
    println!("Record {}: {} ref, {} det, {:.2}% sens, {:.2}% PPV, {} FP",
        record, ref_samples.len(), detected.len(), sensitivity, ppv, false_pos);

    println!("\nTP: {}, FN: {}, FP: {}", true_pos, false_neg, false_pos);
    println!("Sensitivity: {:.2}%", sensitivity);
    println!("PPV: {:.2}%", ppv);

    println!("\nFalse Negatives (missed, first 20): {:?}", &false_negatives[..20.min(false_negatives.len())]);
    println!("\nFalse Positives (extra, first 20): {:?}", &false_positives[..20.min(false_positives.len())]);

    // Check FP clustering - gaps between consecutive FPs
    if false_positives.len() > 1 {
        let mut fp_gaps: Vec<u32> = Vec::new();
        for i in 1..false_positives.len() {
            fp_gaps.push(false_positives[i] - false_positives[i-1]);
        }
        let clustered = fp_gaps.iter().filter(|&&g| g < 500).count(); // < 500 samples = ~1.4s
        println!("\nFP clustering: {} of {} gaps are <500 samples (clustered)", clustered, fp_gaps.len());
    }

    // FPs by rhythm state
    println!("\n=== FPs by Rhythm State ===");
    let mut fps_by_state: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for fp in &false_positives {
        // Find which rhythm state this FP falls into
        let mut current_state = "unknown".to_string();
        for (state, sample) in &state_changes {
            if *sample <= *fp {
                current_state = state.clone();
            } else {
                break;
            }
        }
        *fps_by_state.entry(current_state).or_insert(0) += 1;
    }
    let mut state_counts: Vec<_> = fps_by_state.iter().collect();
    state_counts.sort_by(|a, b| b.1.cmp(a.1)); // sort by count descending
    for (state, count) in state_counts {
        println!("{:10} {:4} FPs", state, count);
    }

    // Also show FNs by rhythm state
    println!("\n=== FNs by Rhythm State ===");
    let mut fns_by_state: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for fn_sample in &false_negatives {
        let mut current_state = "unknown".to_string();
        for (state, sample) in &state_changes {
            if *sample <= *fn_sample {
                current_state = state.clone();
            } else {
                break;
            }
        }
        *fns_by_state.entry(current_state).or_insert(0) += 1;
    }
    let mut fn_state_counts: Vec<_> = fns_by_state.iter().collect();
    fn_state_counts.sort_by(|a, b| b.1.cmp(a.1));
    for (state, count) in fn_state_counts {
        println!("{:10} {:4} FNs", state, count);
    }

    // Find near-miss pairs (55-70 samples apart)
    println!("\n=== Near-miss pairs (55-70 samples apart) ===");
    for fn_sample in &false_negatives {
        for fp_sample in &false_positives {
            let diff = (*fn_sample as i64 - *fp_sample as i64).abs();
            if diff >= 55 && diff <= 70 {
                println!("Ref: {} → Det: {} | gap: {} samples = {:.1}ms | det is {} samples EARLY",
                    fn_sample, fp_sample, diff, diff as f64 * 1000.0 / 360.0,
                    *fn_sample as i64 - *fp_sample as i64);
                break;
            }
        }
    }

    // Analyze offset by beat type
    println!("\n=== Offset by Beat Type ===");
    let mut offsets_by_type: HashMap<String, Vec<i64>> = HashMap::new();

    // For each reference beat, find closest detected beat within 100 samples
    for ref_beat in &ref_beats {
        let mut best_det: Option<u32> = None;
        let mut best_diff: i64 = 101;

        for det in &detected {
            let diff = (*det as i64 - ref_beat.sample as i64).abs();
            if diff < best_diff {
                best_diff = diff;
                best_det = Some(*det);
            }
        }

        if let Some(det) = best_det {
            if best_diff <= 100 {
                let offset = det as i64 - ref_beat.sample as i64;
                let type_name = format!("{:?}", ref_beat.bt);
                offsets_by_type.entry(type_name).or_insert_with(Vec::new).push(offset);
            }
        }
    }

    // Print stats per type
    let mut types: Vec<_> = offsets_by_type.keys().cloned().collect();
    types.sort();

    for bt in types {
        let offsets = &offsets_by_type[&bt];
        let count = offsets.len();
        let mean: f64 = offsets.iter().sum::<i64>() as f64 / count as f64;
        let min = offsets.iter().min().unwrap();
        let max = offsets.iter().max().unwrap();

        // Count how many are within tolerance
        let within_tol = offsets.iter().filter(|o| o.abs() <= tolerance as i64).count();

        println!("{:25} n={:4}  mean={:+6.1}  range=[{:+4}, {:+4}]  within±54: {}%",
            bt, count, mean, min, max, 100 * within_tol / count);
    }

    // Detailed diagnostics for failures outside ±54 tolerance
    println!("\n=== Detailed Failure Diagnostics (outside ±54) ===");
    let (signal, _) = wfdb::record_212_decode(&dat_path);
    let distance = detector::distance_qrs(&signal);
    let median = detector::count_samples_median(&distance);
    let window_size: u32 = 43;
    let threshold = 4.0 * median;

    let mut failure_count = 0;
    for ref_beat in &ref_beats {
        if failure_count >= 3 { break; } // Only show 3 failures

        // Find closest detected beat
        let mut best_det: Option<u32> = None;
        let mut best_diff: i64 = i64::MAX;
        for det in &detected {
            let diff = (*det as i64 - ref_beat.sample as i64).abs();
            if diff < best_diff {
                best_diff = diff;
                best_det = Some(*det);
            }
        }

        // Only analyze if outside tolerance
        if best_diff > 54 && best_diff < 150 {
            failure_count += 1;
            let ref_s = ref_beat.sample;

            println!("\n--- Failure #{} ({:?}) ---", failure_count, ref_beat.bt);
            println!("Reference R-peak: sample {}", ref_s);
            if let Some(det) = best_det {
                println!("Closest detection: sample {} (offset: {:+})", det, det as i64 - ref_s as i64);
            } else {
                println!("No detection found nearby");
            }

            // Find where energy threshold was crossed near this beat
            let search_start = if ref_s > 100 { ref_s - 100 } else { 0 };
            let search_end = (ref_s + 100).min(distance.len() as u32 - window_size);

            println!("\nEnergy threshold: {:.0}", threshold);
            println!("Searching for threshold crossings in [{}, {}]:", search_start, search_end);

            for curr_dist in search_start..search_end {
                let mut energy: u64 = 0;
                for i in curr_dist..curr_dist + window_size {
                    if (i as usize) < distance.len() {
                        energy += distance[i as usize] as u64;
                    }
                }
                if energy as f64 > threshold {
                    let spike_window_start = curr_dist + 40;
                    let spike_window_end = spike_window_start + 43;
                    println!("  Threshold crossed at {}: energy={} (check_spike window: [{}, {}])",
                        curr_dist, energy, spike_window_start, spike_window_end);
                    break; // Just show first crossing
                }
            }

            // Print raw ECG around the reference beat
            let ecg_start = if ref_s > 80 { ref_s - 80 } else { 0 };
            let ecg_end = (ref_s + 80).min(signal.len() as u32);
            println!("\nRaw ECG around ref beat (sample: value):");
            print!("  ");
            for s in ecg_start..ecg_end {
                if s == ref_s {
                    print!("[{}:{}] ", s, signal[s as usize]); // highlight ref
                } else if s % 20 == 0 || s == ecg_start {
                    print!("{}:{} ", s, signal[s as usize]);
                }
            }
            println!();
        }
    }

    // Compare duplicate FP pairs vs legitimate short RR pairs
    println!("\n=== DUPLICATE FP PAIRS (bad - same beat detected twice) ===");

    // Find duplicate FP pairs
    let mut fps_by_ref: HashMap<u32, Vec<u32>> = HashMap::new();
    for (j, &det) in detected.iter().enumerate() {
        if matched_det[j] { continue; }
        let mut nearest_ref: u32 = 0;
        let mut nearest_diff: i64 = i64::MAX;
        for &ref_s in &ref_samples {
            let diff = (det as i64 - ref_s as i64).abs();
            if diff < nearest_diff {
                nearest_diff = diff;
                nearest_ref = ref_s;
            }
        }
        if nearest_diff <= 200 {
            fps_by_ref.entry(nearest_ref).or_insert_with(Vec::new).push(det);
        }
    }

    let mut dup_count = 0;
    for (_ref_s, dets) in &fps_by_ref {
        if dets.len() < 2 || dup_count >= 10 { continue; }
        dup_count += 1;

        let mut sorted = dets.clone();
        sorted.sort();
        let det1 = sorted[0];
        let det2 = sorted[1];
        let gap = det2 - det1;

        println!("\n--- Duplicate #{}: det {} → det {} (gap {} samples, {:.0}ms) ---",
            dup_count, det1, det2, gap, gap as f64 * 1000.0 / 360.0);

        // Energy trace between detections
        print!("Energy between: ");
        let mut energies: Vec<u64> = Vec::new();
        for pos in det1..det2 {
            let mut e: u64 = 0;
            for i in pos..pos + window_size {
                if (i as usize) < distance.len() {
                    e += distance[i as usize] as u64;
                }
            }
            energies.push(e);
        }
        let min_e = energies.iter().min().unwrap_or(&0);
        let max_e = energies.iter().max().unwrap_or(&0);
        let drops_below = energies.iter().any(|&e| (e as f64) < threshold);
        println!("min={} max={} drops_below_thresh={}", min_e, max_e, drops_below);

        // Raw ECG between detections
        print!("ECG [{}-{}]: ", det1, det2);
        let ecg_min = signal[det1 as usize..det2 as usize].iter().min().unwrap_or(&0);
        let ecg_max = signal[det1 as usize..det2 as usize].iter().max().unwrap_or(&0);
        let ecg_range = *ecg_max - *ecg_min;
        println!("min={} max={} range={}", ecg_min, ecg_max, ecg_range);
    }

    // Find legitimate short RR pairs (consecutive ref beats with small gap, both detected)
    println!("\n=== LEGITIMATE SHORT RR PAIRS (good - two real beats close together) ===");

    let mut legit_count = 0;
    for i in 1..ref_samples.len() {
        if legit_count >= 10 { break; }

        let rr_gap = ref_samples[i] - ref_samples[i-1];
        if rr_gap < 250 || rr_gap > 350 { continue; } // Look for RR ~180-280ms

        // Check if both were detected (matched)
        if !matched_ref[i-1] || !matched_ref[i] { continue; }

        // Find the actual detections for these refs
        let ref1 = ref_samples[i-1];
        let ref2 = ref_samples[i];

        let mut det1: Option<u32> = None;
        let mut det2: Option<u32> = None;
        for &d in &detected {
            if (d as i64 - ref1 as i64).abs() <= tolerance as i64 { det1 = Some(d); }
            if (d as i64 - ref2 as i64).abs() <= tolerance as i64 { det2 = Some(d); }
        }

        if det1.is_none() || det2.is_none() { continue; }
        let d1 = det1.unwrap();
        let d2 = det2.unwrap();
        if d2 <= d1 { continue; }

        legit_count += 1;
        let gap = d2 - d1;

        println!("\n--- Legit #{}: det {} → det {} (gap {} samples, {:.0}ms) ---",
            legit_count, d1, d2, gap, gap as f64 * 1000.0 / 360.0);

        // Energy trace between detections
        let mut energies: Vec<u64> = Vec::new();
        for pos in d1..d2 {
            let mut e: u64 = 0;
            for i in pos..pos + window_size {
                if (i as usize) < distance.len() {
                    e += distance[i as usize] as u64;
                }
            }
            energies.push(e);
        }
        let min_e = energies.iter().min().unwrap_or(&0);
        let max_e = energies.iter().max().unwrap_or(&0);
        let drops_below = energies.iter().any(|&e| (e as f64) < threshold);
        println!("Energy between: min={} max={} drops_below_thresh={}", min_e, max_e, drops_below);

        // Raw ECG between detections
        if d2 > d1 && (d2 as usize) < signal.len() {
            let ecg_min = signal[d1 as usize..d2 as usize].iter().min().unwrap_or(&0);
            let ecg_max = signal[d1 as usize..d2 as usize].iter().max().unwrap_or(&0);
            let ecg_range = *ecg_max - *ecg_min;
            println!("ECG [{}-{}]: min={} max={} range={}", d1, d2, ecg_min, ecg_max, ecg_range);
        }
    }
}

