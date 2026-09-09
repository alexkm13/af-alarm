use crate::af::{NaiveDetector, CusumDetector};

pub struct RRProcessor {
    pub prev_beat: Option<u32>,
    pub prev_rr: Option<u32>,
    pub rr_interval: Vec<(u32, u32)>,
    pub irregularity: Vec<(f64, u32)>, 

    pub naive: NaiveDetector,
    pub cusum: CusumDetector,
}

impl RRProcessor {
    pub fn stream_beat(&mut self, beat: u32) -> () {
        if let Some(b) = self.prev_beat {
            let rr = beat - b;
            self.rr_interval.push((rr, beat));

            if let Some(prev_rr) = self.prev_rr {
                let z_t = (rr as f64 - prev_rr as f64).abs() / rr as f64;
                self.irregularity.push((z_t, beat));

                self.naive.update(z_t, beat);
                self.cusum.update(z_t, beat);
            }
            self.prev_rr = Some(rr);
        }
        self.prev_beat = Some(beat);
    }

    pub fn new() -> Self {
        RRProcessor {
            prev_beat: None,
            prev_rr: None,
            rr_interval: Vec::new(),
            irregularity: Vec::new(),
            naive: NaiveDetector::new(0.3),      
            cusum: CusumDetector::new(0.1, 2.0),         }
    }
}

