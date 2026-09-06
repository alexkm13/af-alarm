use crate::wfdb::{parse_annotations, Beat};

pub fn create_rr_list(file_name: &str) -> Vec<(u32, u32)> {
   let (beats, _states) = parse_annotations(file_name);
   let mut differences: Vec<(u32, u32)> = Vec::new();

   for i in 1..beats.len() {
       let diff = beats[i].sample - beats[i - 1].sample;
       differences.push((diff, beats[i].sample));
   }

   differences
}
