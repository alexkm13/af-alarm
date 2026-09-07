use crate::wfdb::{record_212_decode, parse_annotations};
use std::collections::VecDeque;

pub fn distance_qrs(channel: &Vec<i16>) -> Vec<u32> {
    let mut sq_distance: Vec<u32> = Vec::new();
    
    for i in 1..channel.len() {
        let diff = channel[i] - channel[i - 1];
        sq_distance.push((diff as i32).pow(2) as u32);
    }
    sq_distance
}

pub fn count_samples_median(distance: &Vec<u32>) -> f64 {
    let mut samples = 43;
    let mut window_count = 0;
    let mut windows: Vec<u32> = Vec::new();
    let mut curr_win = 0;
    let mut median = 0.0;
    let window_size = 43;

    

    while samples < 360 {
        for j in 0 + window_count..window_size + window_count {
            curr_win += distance[j];
        }
        windows.push(curr_win);
        curr_win = 0;
        window_count += 1;
        samples += 1;
    }

    windows.sort();

    let mid = windows.len() / 2;
    if windows.len() % 2 == 0 {
        median = (windows[mid - 1] as f64 + windows[mid] as f64) / 2.0;
    } else {
        median = windows[mid] as f64;
    }

    median
}

pub fn check_spike(raw_beats: &Vec<i16>, location: u32) -> Option<u32> {
    // check baselines around window
    // get median of baselines
    // find greatest absolute deviation from median
    // if none, return None, else return location of spike
    let mut curr_largest_distance: f64 = 0.0;
    let mut baseline: Vec<i16> = Vec::new();
    let mut median: f64 = 0.0;
    let mut index: u32 = 0;
    let end_window = location + 43;

    // check and see if both baselines have the necessary space
    if end_window + 14 <= raw_beats.len() as u32 {
        // left baseline
        for i in (location - 14)..location {
            baseline.push(raw_beats[i as usize]);
        }
        // right baseline
        for j in end_window..(end_window) + 14 {
            baseline.push(raw_beats[j as usize]);
        }
    } else {
        // borrow from left to compensate for missing right samples
        let missing = (end_window + 14) - raw_beats.len() as u32;
        if missing > 14 || location < 14 + missing {
            return None;
        }
        for m in (location - (14 + missing))..location {
            baseline.push(raw_beats[m as usize]);
        }
        for n in end_window..raw_beats.len() as u32 {
            baseline.push(raw_beats[n as usize]);
        }
    }

    baseline.sort(); 
    let mid = baseline.len() / 2;
    median = (baseline[mid - 1] as f64 + baseline[mid] as f64) / 2.0;

    for b in location..(end_window) {
        if (raw_beats[b as usize] as f64 - median).abs() > curr_largest_distance {
            curr_largest_distance = (raw_beats[b as usize] as f64 - median).abs();
            index = b;
        }
    }
    
    if curr_largest_distance != 0.0 {
        return Some(index);
    } else {
        return None;
    } 
}

fn min_energy_between(distance: &Vec<u32>, start: u32, end: u32, window_size: u32) -> u64 {
    let mut min_energy: u64 = u64::MAX;
    if end < window_size {
        return min_energy;
    }

    for position in start..(end - window_size) {
        if position + window_size > distance.len() as u32 {
            break;
        }
        let mut energy: u64 = 0;
        for i in position..position + window_size {
            energy += distance[i as usize] as u64;
        }
        if energy < min_energy {
            min_energy = energy;
        }
    }
    min_energy
}

pub fn detector(file_name: &str) -> Vec<u32> {
    let (beats_a, _beats_b) = record_212_decode(file_name);
    let distance = distance_qrs(&beats_a);
    let mut beats: Vec<u32> = Vec::new();
    let mut curr_sum: u64 = 0;
    let window_size: u32 = 43;
    let mut last_beat: u32 = 0;
    let median = count_samples_median(&distance);
    let mut curr_dist: u32 = 360;
    let energy_drop_threshold: u64 = (2.5 * median) as u64;

    while curr_dist <= distance.len() as u32 - window_size {
        for i in curr_dist..curr_dist + window_size {
            curr_sum += distance[i as usize] as u64;
        }

        if (curr_sum as f64) > (4.0 * median) {
            if curr_dist - last_beat > 72 {
                let energy_dropped = last_beat == 0 ||
                    min_energy_between(&distance, last_beat, curr_dist, window_size) < energy_drop_threshold;

                if energy_dropped {
                    if let Some(beat) = check_spike(&beats_a, curr_dist + 40) {
                        curr_dist = beat;
                        last_beat = beat;
                        beats.push(beat);
                    }
                }
            }
        }

        curr_dist += 1;
        curr_sum = 0;
    }
    beats
}
