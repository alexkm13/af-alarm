use crate::wfdb::{parse_annotations, Beat};

pub fn create_rr_list(file_name: &str, fs: u32) -> Vec<(f32, u32)> {
   let (beats, _states) = parse_annotations(file_name);
   let mut differences: Vec<(u32, u32)> = Vec::new();

   for i in 1..beats.len() {
       let diff = beats[i].sample - beats[i - 1].sample;
       let quotient = (diff / fs) as f32;
       differences.push((quotient, beats[i].sample));
   }

   differences
}

pub struct RRProcessor {
    prev_beat: Option<u32>,
    rr_interval: Vec<(u32, u32)>,
}

impl RRProcessor {
    pub fn stream_beat(&mut self, beat: u32) -> () {
        if let Some(b) = self.prev_beat {
            self.rr_interval.push((beat - b, beat));
        }
        self.prev_beat = Some(beat);
    }
}

