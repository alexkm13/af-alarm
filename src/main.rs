mod wfdb;
mod rr;
use wfdb::{BeatType, Beat};

fn main() {
    let (beats, state_changes) = wfdb::parse_annotations("data/mit-bih-arrhythmia-database-1.0.0/200.atr");
    let rr_intervals = rr::create_rr_list("data/mit-bih-arrhythmia-database-1.0.0/200.atr");

    // Count beats by type
    let normal = beats.iter().filter(|b| b.bt == BeatType::Normal).count();
    let pvc = beats.iter().filter(|b| b.bt == BeatType::PVC).count();
    let fusion = beats.iter().filter(|b| b.bt == BeatType::Fusion).count();

    println!("Total beats: {}", beats.len());
    println!("Normal: {}, PVC: {}, Fusion: {}", normal, pvc, fusion);
    println!("Last beat: {:?}", beats.last());
    println!("State changes: {}", state_changes.len());

    // Verify RR intervals
    println!("\nRR intervals count: {}", rr_intervals.len());
    println!("First 5 RR intervals: {:?}", &rr_intervals[..5]);

    // Manual verification: RR[0] should be beats[1].sample - beats[0].sample
    println!("\nManual check:");
    println!("beats[0].sample = {}, beats[1].sample = {}", beats[0].sample, beats[1].sample);
    println!("Expected RR[0] = {}", beats[1].sample - beats[0].sample);
}

