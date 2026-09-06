use crate::wfdb::{parse_annotations, Beat};

pub fn create_rr_list(file_name: &str, fs: u32) -> Vec<(f32, u32)> {
   let (beats, _states) = parse_annotations(file_name);
   let mut differences: Vec<(f32, u32)> = Vec::new();

   for i in 1..beats.len() {
       let diff = beats[i].sample - beats[i - 1].sample;
       let quotient = (diff / fs) as f32;
       differences.push((quotient, beats[i].sample));
   }

   differences
}
