# AF Alarm

A low-latency atrial fibrillation alarm pipeline written in Rust.

AF Alarm streams detected ECG beats through a bounded SPSC queue, computes RR intervals online, and compares a naive irregularity threshold against CUSUM for AF detection.

The main question:

> At approximately matched false-alarm rates, can CUSUM reduce AF detection delay or miss rate compared with a naive threshold using the same irregularity signal?

> Note: This is an experimental research project.

## Pipeline

```text
ECG
 → beat detection
 → bounded SPSC queue
 → streaming RR intervals
 → irregularity score
 → threshold / CUSUM
 → AF alarm
```

The runtime is written in Rust. Python is used only for offline evaluation and plotting.

## Features

* WFDB `.hea`, `.dat` (212), and `.atr` parsing
* custom QRS / beat detector
* bounded lock-free SPSC queue
* Acquire/Release publication between producer and consumer
* explicit backpressure with `park` / `unpark`
* streaming RR computation
* naive threshold and CUSUM alarm strategies
* reference-beat vs detected-beat evaluation
* false-alarm, detection-delay, and miss-rate measurement

## SPSC

Beat events are passed between one producer and one consumer through fixed preallocated storage.

Queue occupancy is derived from producer and consumer progress:

```text
occupancy = pushes - pops
```

with:

```text
0 <= pushes - pops <= capacity
```

If the queue is full, the producer retains the beat and waits until there is space.

## Evaluation

Two pipelines are compared:

```text
reference beats → RR → AF detector
```

and:

```text
ECG → detected beats → RR → AF detector
```

This separates AF-detector limitations from upstream beat-detection errors.

Reported metrics:

* false alarms / non-AF hour
* AF detection delay
* miss rate

CUSUM and the naive detector are compared at approximately matched false-alarm burden.

## Clinical framing

AF commonly produces irregular ventricular timing, but RR irregularity is not specific to AF.

Important confounders include:

* PACs / PVCs and frequent ectopy
* sinus arrhythmia
* other supraventricular rhythms
* abrupt heart-rate changes
* missed or double-counted QRS complexes
* motion / electrode artifact

AF episode boundaries follow rhythm annotations: onset begins at a transition to `(AFIB`, and offset is the next transition away from `(AFIB`.

A 30-second minimum episode length may be explored experimentally, but is not treated as a universal clinical threshold.

The project targets detection on the order of **tens of seconds** while balancing miss rate against cumulative false-alarm burden.

## Build

```bash
cargo build --release
cargo test
cargo run --release
```

## Results

Evaluated on MIT-BIH record 201, which contains 3 AF episodes (378s, 37s, 191s).

### Default Parameters

| Input           | Detector           | FAR / hr | Detection delay | Miss rate |
| --------------- | ------------------ | -------: | --------------: | --------: |
| Reference beats | Naive (thresh=0.3) |    528.4 |            3.5s |        0% |
| Reference beats | CUSUM (k=0.1,h=2)  |      0.0 |           15.6s |       67% |
| Detected beats  | Naive (thresh=0.3) |    606.4 |            2.6s |        0% |
| Detected beats  | CUSUM (k=0.1,h=2)  |      0.0 |           15.6s |       67% |

### Pareto Frontier (Reference Path)

**Naive Detector:**

| Threshold | Sensitivity | FAR / hr | Delay |
| --------: | ----------: | -------: | ----: |
|      0.90 |       100%  |    261.2 | 38.5s |
|      0.80 |       100%  |    273.2 | 16.6s |
|      0.70 |       100%  |    279.2 | 12.8s |
|      0.60 |       100%  |    282.2 |  5.0s |
|      0.10 |       100%  |    396.3 |  1.9s |

**CUSUM Detector:**

| k    | h   | Sensitivity | FAR / hr | Delay |
| ---: | --: | ----------: | -------: | ----: |
| 0.05 | 0.5 |        33%  |      0.0 |  3.8s |

### Observations

The naive threshold achieves 100% sensitivity across all tested thresholds but with high false-alarm rates (261-579/hr). CUSUM with current parameters achieves zero false alarms but only detects 1/3 episodes (33% sensitivity).

At matched FAR comparison is not directly possible since CUSUM operates at FAR=0 while naive operates at FAR>260/hr. The detectors occupy different regions of the sensitivity/FAR tradeoff space.

Detected-beat path shows ~15% FAR degradation vs reference path for naive detector (606 vs 528/hr), indicating upstream beat-detection errors contribute additional irregularity.

## Limitations

AF Alarm is an RR-irregularity warning system, not a clinical AF diagnostic system.

RR irregularity can be caused by rhythms other than AF, as well as QRS-detection and measurement errors. The custom beat detector is intentionally simple, and the system has not been clinically validated.

## Acknowledgments

Biomedical-engineering input helped define the physiological confounders, AF episode semantics, expected failure modes, alarm-fatigue considerations, and interpretation of the false-alarm / detection-delay tradeoff.

Engineering design, implementation details, failures, and lessons are documented in [`POSTMORTEM.md`](POSTMORTEM.md).

MIT License
