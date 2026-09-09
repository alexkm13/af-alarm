pub struct NaiveDetector {
    pub threshold: f64,
    pub in_alarm: bool,
    pub alarms: Vec<u32>,  
}

impl NaiveDetector {
    pub fn new(threshold: f64) -> Self {
        NaiveDetector {
            threshold,
            in_alarm: false,
            alarms: Vec::new(),
        }
    }

    pub fn update(&mut self, z_t: f64, beat: u32) {
        if z_t > self.threshold {
            if !self.in_alarm {
                self.alarms.push(beat);
                self.in_alarm = true;
            }
        } else {
            self.in_alarm = false;
        }
    }
}

pub struct CusumDetector {
    pub k: f64,       
    pub h: f64,      
    pub s_t: f64,     
    pub in_alarm: bool,
    pub alarms: Vec<u32>,
}

impl CusumDetector {
    pub fn new(k: f64, h: f64) -> Self {
        CusumDetector {
            k,
            h,
            s_t: 0.0,
            in_alarm: false,
            alarms: Vec::new(),
        }
    }

    pub fn update(&mut self, z_t: f64, beat: u32) {
        self.s_t = (self.s_t + z_t - self.k).max(0.0);

        if self.s_t > self.h {
            if !self.in_alarm {
                self.alarms.push(beat);
                self.in_alarm = true;
            }
        } else {
            self.in_alarm = false;
        }
    }
}
