mod wfdb;
use wfdb::{BeatType, Beat};

fn main() {
    let (beats, state_changes) = wfdb::parse_annotations("data/mit-bih-arrhythmia-database-1.0.0/200.atr");

    // Count beats by type
    let normal = beats.iter().filter(|b| b.bt == BeatType::Normal).count();
    let pvc = beats.iter().filter(|b| b.bt == BeatType::PVC).count();
    let fusion = beats.iter().filter(|b| b.bt == BeatType::Fusion).count();

    println!("Total beats: {}", beats.len());
    println!("Normal: {}, PVC: {}, Fusion: {}", normal, pvc, fusion);
    println!("Last beat: {:?}", beats.last());
    println!("State changes: {}", state_changes.len());
}

