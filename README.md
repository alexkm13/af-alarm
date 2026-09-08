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

| Input           | Detector  | FAR / hr | Detection delay | Miss rate |
| --------------- | --------- | -------: | --------------: | --------: |
| Reference beats | Threshold |     TODO |            TODO |      TODO |
| Reference beats | CUSUM     |     TODO |            TODO |      TODO |
| Detected beats  | Threshold |     TODO |            TODO |      TODO |
| Detected beats  | CUSUM     |     TODO |            TODO |      TODO |

## Limitations

AF Alarm is an RR-irregularity warning system, not a clinical AF diagnostic system.

RR irregularity can be caused by rhythms other than AF, as well as QRS-detection and measurement errors. The custom beat detector is intentionally simple, and the system has not been clinically validated.

## Acknowledgments

Biomedical-engineering input helped define the physiological confounders, AF episode semantics, expected failure modes, alarm-fatigue considerations, and interpretation of the false-alarm / detection-delay tradeoff.

Engineering design, implementation details, failures, and lessons are documented in [`POSTMORTEM.md`](POSTMORTEM.md).

MIT License
