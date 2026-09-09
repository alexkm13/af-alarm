use crate::rr::RRProcessor;
use crate::wfdb::{parse_annotations, Beat};

const SAMPLE_RATE: u32 = 360;


#[derive(Debug, Clone)]
pub struct AFEpisode {
    pub start_sample: u32,
    pub end_sample: u32,
}

impl AFEpisode {
    pub fn duration_seconds(&self) -> f64 {
        (self.end_sample - self.start_sample) as f64 / SAMPLE_RATE as f64
    }
}

pub fn extract_af_episodes(state_changes: &[(String, u32)], record_end: u32) -> Vec<AFEpisode> {
    let mut episodes: Vec<AFEpisode> = Vec::new();
    let mut af_start: Option<u32> = None;

    for (state, sample) in state_changes {
        if state.contains("AFIB") {
            if af_start.is_none() {
                af_start = Some(*sample);
            }
        } else {
            if let Some(start) = af_start {
                episodes.push(AFEpisode {
                    start_sample: start,
                    end_sample: *sample,
                });
                af_start = None;
            }
        }
    }

    if let Some(start) = af_start {
        episodes.push(AFEpisode {
            start_sample: start,
            end_sample: record_end,
        });
    }

    episodes
}


pub fn filter_episodes_by_duration(episodes: &[AFEpisode], min_seconds: f64) -> Vec<AFEpisode> {
    episodes
        .iter()
        .filter(|e| e.duration_seconds() >= min_seconds)
        .cloned()
        .collect()
} 


#[derive(Debug)]
pub struct EvalResult {
    pub detector_name: String,
    pub total_af_episodes: usize,
    pub detected_episodes: usize,
    pub missed_episodes: usize,
    pub false_alarms: usize,
    pub non_af_hours: f64,
    pub false_alarm_rate: f64,  
    pub detection_delays: Vec<f64>,  
    pub mean_delay: f64,
}


fn is_in_af(alarm_sample: u32, episodes: &[AFEpisode]) -> bool {
    episodes.iter().any(|e| alarm_sample >= e.start_sample && alarm_sample < e.end_sample)
}


fn detection_delay(episode: &AFEpisode, alarms: &[u32]) -> Option<f64> {
    for &alarm in alarms {
        if alarm >= episode.start_sample && alarm < episode.end_sample {
            return Some((alarm - episode.start_sample) as f64 / SAMPLE_RATE as f64);
        }
    }
    None
}


pub fn evaluate_detector(
    name: &str,
    alarms: &[u32],
    af_episodes: &[AFEpisode],
    record_duration_samples: u32,
) -> EvalResult {
    
    let mut detected = 0;
    let mut detection_delays: Vec<f64> = Vec::new();

    for episode in af_episodes {
        if let Some(delay) = detection_delay(episode, alarms) {
            detected += 1;
            detection_delays.push(delay);
        }
    }

    
    let false_alarms = alarms.iter().filter(|&&a| !is_in_af(a, af_episodes)).count();


    let total_af_samples: u32 = af_episodes.iter().map(|e| e.end_sample - e.start_sample).sum();
    let non_af_samples = record_duration_samples.saturating_sub(total_af_samples);
    let non_af_hours = non_af_samples as f64 / SAMPLE_RATE as f64 / 3600.0;

    let far = if non_af_hours > 0.0 {
        false_alarms as f64 / non_af_hours
    } else {
        0.0
    };

    let mean_delay = if !detection_delays.is_empty() {
        detection_delays.iter().sum::<f64>() / detection_delays.len() as f64
    } else {
        f64::NAN
    };

    EvalResult {
        detector_name: name.to_string(),
        total_af_episodes: af_episodes.len(),
        detected_episodes: detected,
        missed_episodes: af_episodes.len() - detected,
        false_alarms,
        non_af_hours,
        false_alarm_rate: far,
        detection_delays,
        mean_delay,
    }
}


pub fn run_reference_path(atr_path: &str) -> (RRProcessor, Vec<AFEpisode>, u32) {
    let (beats, state_changes) = parse_annotations(atr_path);
    let record_end = beats.last().map(|b| b.sample).unwrap_or(0);

    let mut processor = RRProcessor::new();
    for beat in &beats {
        processor.stream_beat(beat.sample);
    }

    let af_episodes = extract_af_episodes(&state_changes, record_end);

    (processor, af_episodes, record_end)
}

#[derive(Debug)]
pub struct PathComparison {
    pub ref_eval: EvalResult,
    pub det_eval: EvalResult,
    pub far_degradation: f64,  
    pub delay_degradation: f64,  
}

pub fn compare_paths(
    ref_processor: &RRProcessor,
    det_processor: &RRProcessor,
    af_episodes: &[AFEpisode],
    record_duration: u32,
    detector_name: &str,
) -> PathComparison {
    let (ref_alarms, det_alarms) = match detector_name {
        "naive" => (&ref_processor.naive.alarms, &det_processor.naive.alarms),
        "cusum" => (&ref_processor.cusum.alarms, &det_processor.cusum.alarms),
        _ => panic!("Unknown detector: {}", detector_name),
    };

    let ref_eval = evaluate_detector(&format!("{}_ref", detector_name), ref_alarms, af_episodes, record_duration);
    let det_eval = evaluate_detector(&format!("{}_det", detector_name), det_alarms, af_episodes, record_duration);

    let far_degradation = if ref_eval.false_alarm_rate > 0.0 {
        det_eval.false_alarm_rate / ref_eval.false_alarm_rate
    } else if det_eval.false_alarm_rate > 0.0 {
        f64::INFINITY
    } else {
        1.0
    };

    let delay_degradation = det_eval.mean_delay - ref_eval.mean_delay;

    PathComparison {
        ref_eval,
        det_eval,
        far_degradation,
        delay_degradation,
    }
}
