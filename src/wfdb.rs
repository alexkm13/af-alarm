use std::fs;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeatType {
    Normal,
    LeftBundleBranchBlock,
    RightBundleBranchBlock,
    BundleBranchBlock,
    AtrialPremature,
    AberratedAtrialPremature,
    JunctionalPremature,
    SupraventricularPremature,
    PVC,
    RonTPVC,
    Fusion,
    AtrialEscape,
    JunctionalEscape,
    SupraventricularEscape,
    VentricularEscape,
    Paced,
    PacedFusion,
    Unclassifiable,
    UnclassifiedLearning,
    Unknown(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Beat {
    pub sample: u32,
    pub bt: BeatType,
}

pub fn parse_record_line() -> io::Result<()> {
    let contents = fs::read_to_string("data/100.hea")?;
    let mut lines = contents.lines();
    let first_line = lines.next().unwrap();
    let mut split_first_line = first_line.split_whitespace();
    
    let record_name = split_first_line.next().unwrap();
    let num_signals: u32 = split_first_line.next().unwrap().parse().unwrap();
    let sample_rate: u32 = split_first_line.next().unwrap().parse().unwrap();
    let num_samples: u32 = split_first_line.next().unwrap().parse().unwrap();

    println!(
        "{} {} {} {}",
        record_name, num_signals, sample_rate, num_samples
    );

    Ok(())
}

pub fn read_signals(lines: &mut std::str::Lines<'_>, num_signals: u32) -> () {
    // iterate through # of lines that are signal
    for _ in 0..num_signals {
        let mut curr_file = lines.next().unwrap().split_whitespace();
        // read first two elements of signal line
        let file_name = curr_file.next().unwrap();
        let format: u32 = curr_file.next().unwrap().parse().unwrap();

        println!("{} {}", file_name, format)
    }
}

pub fn decode_212_ecg(bytes: &[u8; 3]) -> (i16, i16) {
    // decompose 3 bytes to 12-bit ints
    let mut int_a: i16 = bytes[0] as i16 + (((bytes[1] & 0x0F) as i16) << 8);
    let mut int_b: i16 = bytes[2] as i16 + (((bytes[1] & 0xF0) as i16) << 4);
    
    // interpret reconstructed 12-bit values as signed two's-complement
    if int_a >= 2048 {
        int_a -= 4096;
    } 

    if int_b >= 2048 {
        int_b -= 4096;
    }
    
    (int_a, int_b)
}

pub fn record_212_decode(data_path: &str)-> (Vec<i16>, Vec<i16>) {
    let dat = fs::read(data_path).unwrap();
    let mut a: Vec<i16> = Vec::new();
    let mut b: Vec<i16> = Vec::new();
    for i in (0..dat.len()).step_by(3) {
        let bytes: [u8; 3] = [dat[i], dat[i + 1], dat[i + 2]];

        let (int_a, int_b) = decode_212_ecg(&bytes);
        
        a.push(int_a);
        b.push(int_b);
    }
    (a, b) 

}

pub fn decode_annotation(bytes: &[u8; 2]) -> (u16, u8) {
    // decompose 2 bytes to one 6 bit annontation type and one 10 bit time delta 
    let time_delta: u16 = bytes[0] as u16 + (((bytes[1] & 0x03) as u16) << 8);
    let annotation: u8 = (bytes[1] & 0xFC) as u8 >> 2;
        
    (time_delta, annotation)
}

// processes different types of annotations
pub fn process_annotation(
    a: u16, b: u8, atr: &[u8], i: &mut usize, s: &mut u32,
    num: &mut u32, channel: &mut u32, pending_state_change: &mut Option<u32>,
    state_changes: &mut Vec<(String, u32)>,
) {
    match b {
        28 => {
            *s += a as u32;
            *pending_state_change = Some(*s);
            *i += 2;
        }
        59 => {
            let skip_interval = &atr[*i + 2..*i + 6];
            let high = &skip_interval[0..2];
            let lo = &skip_interval[2..4];

            let hi_word = (high[1] as u16) << 8 | (high[0] as u16);
            let lo_word = (lo[1] as u16) << 8 | (lo[0] as u16);

            let word = (hi_word as u32) << 16 | lo_word as u32;

            *i += 6;
            *s += word;
        }
        60 => {
            *num = a as u32;
            *i += 2;
        }
        61 => {
            *i += 2;
        }
        62 => {
            *channel = a as u32;
            *i += 2;
        }
        63 => {
            if let Some(n) = *pending_state_change {
                let payload = &atr[*i + 2..*i + 2 + a as usize];
                let new_state = std::str::from_utf8(payload).unwrap();

                if let Some(last_element) = state_changes.last() {
                    if last_element.0 != new_state {
                        state_changes.push((new_state.to_string(), n));
                    }
                } else {
                    state_changes.push((new_state.to_string(), n));
                }
                *pending_state_change = None;
            }

            if a % 2 == 1 {
                *i += 2 + a as usize + 1;
            } else {
                *i += 2 + a as usize;
            }
        }
        _ => {
            *s += a as u32;
            *i += 2;
        }
    }
}

pub fn classify_beat(b: u8) -> Option<BeatType> {
    let beat_type = match b {
            1 => Some(BeatType::Normal),
            2 => Some(BeatType::LeftBundleBranchBlock),
            3 => Some(BeatType::RightBundleBranchBlock),
            4 => Some(BeatType::AberratedAtrialPremature),
            5 => Some(BeatType::PVC),
            6 => Some(BeatType::Fusion),
            7 => Some(BeatType::JunctionalPremature),
            8 => Some(BeatType::AtrialPremature),
            9 => Some(BeatType::SupraventricularPremature),
            10 => Some(BeatType::VentricularEscape),
            11 => Some(BeatType::JunctionalEscape),
            12 => Some(BeatType::Paced),
            13 => Some(BeatType::Unclassifiable),
            25 => Some(BeatType::BundleBranchBlock),
            30 => Some(BeatType::UnclassifiedLearning),
            34 => Some(BeatType::AtrialEscape),
            35 => Some(BeatType::SupraventricularEscape),
            38 => Some(BeatType::PacedFusion),
            41 => Some(BeatType::RonTPVC),
            _ => None,
        };
        
    return beat_type; 
}

pub fn parse_annotations(file_name: &str) -> (Vec<Beat>, Vec<(String, u32)>) {
    let atr = fs::read(file_name).unwrap();
    let mut time_delta: Vec<u16> = Vec::new();
    let mut annotations: Vec<u8> = Vec::new();
    let mut beats: Vec<Beat> = Vec::new();
    let mut s: u32 = 0;
    let mut i: usize = 0;
    let mut num: u32 = 0;
    let mut channel: u32 = 0;
    let mut pending_state_change: Option<u32> = None; 
    let mut state_changes: Vec<(String, u32)> = Vec::new(); 

    while i < atr.len() {
        let bytes: [u8; 2] = [atr[i], atr[i + 1]];
 
        let (a, b) = decode_annotation(&bytes);
        if a == 0 && b == 0 { break; }

        process_annotation(a, b, &atr, &mut i, &mut s, 
        &mut num, &mut channel, &mut pending_state_change, 
        &mut state_changes);

        let beat_type = classify_beat(b);
        if let Some(beat) = beat_type {
            let new_beat = Beat {
                sample: s,
                bt: beat,
            };
            beats.push(new_beat);
        }
        time_delta.push(a);
        annotations.push(b);
    }

    (beats, state_changes)
}
