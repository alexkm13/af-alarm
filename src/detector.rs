use crate::wfdb::{record_212_decode};
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

pub fn check_spike(raw_beats: &Vec<u32>, location: u32) -> Option<u32> {
    // check baselines around window
    // get median of baselines
    // find greatest (|val| - median) 
    // if none, return None, else return greatest distance
    let mut spike: u32 = 0;
    let mut curr_largest_distance: u32 = 0;
}

pub fn detector(file_name: &str) -> Vec<u32> {
    let (beats_a, _beats_b) = record_212_decode(file_name);
    let distance = distance_qrs(&beats_a);
    let mut beats: Vec<u32> = Vec::new();
    let mut curr_sum: u64 = 0;
    let window_size: u32 = 43; 
    let mut baseline_median = 0;
    let mut last_beat = 0;
    let median = count_samples_median(&distance);
    let mut curr_dist = 360;

    while curr_dist <= distance.len() as u32 - window_size {
        for i in curr_dist..curr_dist + window_size {
            curr_sum += distance[i] as u64;
        }

        if (curr_sum as f64) > (2.0 * median) {
            if curr_dist - last_beat > 90 {
                // run baseline check
                // if we find spike, update last_beat
                // and put location into beats vec
            }
        }
        
        curr_dist += 1;
        curr_sum = 0;
    } 
}
