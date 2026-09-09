/// Naive threshold detector: alarm when z_t > threshold
/// Counts rising edges only (transition from non-alarm to alarm)
pub struct NaiveDetector {
    pub threshold: f64,
    pub in_alarm: bool,
    pub alarms: Vec<u32>,  // beat samples where alarm triggered
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
                // Rising edge - new alarm
                self.alarms.push(beat);
                self.in_alarm = true;
            }
            // Already in alarm - don't count again
        } else {
            // Below threshold - reset/rearm
            self.in_alarm = false;
        }
    }
}

/// CUSUM detector: S_t = max(0, S_{t-1} + z_t - k)
/// Alarm when S_t > h, rearm when S_t drops back to 0
pub struct CusumDetector {
    pub k: f64,       // slack/allowance
    pub h: f64,       // threshold
    pub s_t: f64,     // cumulative sum
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
        // CUSUM update
        self.s_t = (self.s_t + z_t - self.k).max(0.0);

        if self.s_t > self.h {
            if !self.in_alarm {
                // Rising edge - new alarm
                self.alarms.push(beat);
                self.in_alarm = true;
            }
        } else {
            // Below threshold - rearm
            self.in_alarm = false;
        }
    }
}
