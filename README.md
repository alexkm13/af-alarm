# AF Alarm

A low-latency atrial fibrillation alarm pipeline written in Rust.

AF Alarm streams ECG beat detections through a bounded SPSC queue, computes RR intervals online, and compares a naive irregularity threshold against CUSUM for AF detection.

> **Question:** Can sequential accumulation reduce false alarms or detection delay relative to naive thresholding on the same RR-irregularity signal?

> Experimental research project — not a medical device.

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

The runtime is written in Rust. Python is used for offline evaluation and plotting.

## Features

* WFDB `.hea`, `.dat` (212), and `.atr` parsing
* custom QRS / beat detector
* bounded SPSC queue with Acquire/Release publication
* explicit backpressure with `park` / `unpark`
* streaming RR computation
* naive threshold and CUSUM detectors
* reference-beat vs detected-beat evaluation
* false-alarm, detection-delay, and miss-rate measurement

## Results

Evaluated across **8 MIT-BIH AFDB records**, containing **107 AF episodes** and approximately **1.8 hours of non-AF rhythm**.

The underlying RR-irregularity signal showed substantial overlap between AF and non-AF periods:

| Period | Mean irregularity | Median irregularity |
| ------ | ----------------: | ------------------: |
| AF     |             0.260 |               0.188 |
| Non-AF |             0.222 |               0.079 |

At matched **44.9% episode sensitivity**:

| Detector | Parameters       | FAR / hr | Mean delay |
| -------- | ---------------- | -------: | ---------: |
| Naive    | `threshold=0.90` |    189.4 |      27.6s |
| CUSUM    | `k=0.45, h=0.3`  | **97.7** |  **17.0s** |

At this operating point, CUSUM produced:

* **48% fewer false alarms**
* **10.5s faster detection**
* the same episode sensitivity

However, CUSUM reached a maximum sensitivity of only **44.9%** across the tested parameter grid, while naive thresholding reached **86.9%**.

The main limitation therefore appears to be the **RR-irregularity signal itself**. CUSUM improves discrimination when AF produces persistent irregularity, but many AF episodes do not generate enough sustained evidence to be detected this way.

Absolute false-alarm rates remained high for both approaches.

## SPSC

Beat events pass from one producer to one consumer through fixed preallocated storage.

Queue occupancy is derived from producer and consumer progress:

```text
occupancy = pushes - pops
```

with the invariant:

```text
0 <= pushes - pops <= capacity
```

The producer publishes completed writes with Release ordering; the consumer observes them with Acquire ordering. The reverse handoff protects slot reuse.

When the queue is full, the producer retains the beat and waits rather than dropping or overwriting data.

## Evaluation

Two downstream-identical paths are evaluated:

```text
reference beats
 → RR
 → irregularity
 → detector
```

```text
ECG
 → custom beat detector
 → RR
 → irregularity
 → detector
```

This separates limitations of the AF detector from errors introduced by upstream QRS detection.

Metrics include:

* episode sensitivity / miss rate
* false alarms per non-AF hour
* detection delay

## Clinical framing

AF often produces irregular ventricular timing, but RR irregularity is not specific to AF.

Important confounders include PACs/PVCs, ectopy, sinus arrhythmia, other supraventricular rhythms, abrupt rate changes, QRS detection errors, and motion/electrode artifact.

AF episode boundaries follow the MIT-BIH rhythm annotations: onset begins at `(AFIB`, and offset is the next transition away from `(AFIB`.

Biomedical-engineering input informed these definitions, expected failure modes, alarm-fatigue considerations, and interpretation of the sensitivity / false-alarm / delay tradeoff.

## Build

```bash
cargo build --release
cargo test
cargo run --release
```

## Limitations

AF Alarm is an RR-irregularity warning system, **not a clinical diagnostic system**.

The current study is limited by:

* substantial overlap between AF and non-AF RR irregularity
* a deliberately simple custom QRS detector
* limited non-AF exposure for estimating false-alarm burden
* evaluation on MIT-BIH datasets rather than prospective clinical monitoring
* lack of ECG morphology information in the AF decision rule

## Acknowledgments

Thanks to **[Dan Vu](https://www.linkedin.com/in/dan-vu-b9254b248)** for the biomedical engineering work that informed the clinical framing, AF episode semantics, confounder analysis, and interpretation of the false-alarm / detection-delay tradeoff.

## License

MIT

