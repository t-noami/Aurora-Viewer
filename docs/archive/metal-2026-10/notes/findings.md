# Metal performance repair: working notes

2026-10-30; base b8fee438; Apple M2 Pro, 16 GPU cores, 32 GB, macOS 15.7.7. Windows report from human: RTX 4080 SUPER, 4K display, over 160 FPS; CPU, rendering resolution, settings and scene matching are not verified.

## Confirmed

- wgpu 30.0.1 Metal query reset is a no-op. Multiple frames share the query set, readbacks use three buffers, resolution is before egui. Sample results were aggregated more than once and stale values retained.
- Metal timestamp reproduction runs one GPU submission and blocking readback per iteration only in the test, with no frame-loop wait. On the M2 Pro, serial tests reported render-pass sample pairs where the end is zero or earlier than the start, including a single pass with one encoder and a fresh query set. A shared compute-pass query set produced 27/200 pairs with at least one timestamp not progressing, despite zero reversed pairs. A fresh compute query set produced zero pairs in 117/200 cases. These test cases establish neither the precise underlying fault nor the accuracy of other positive pairs.
- The offline 600-object demo, Immediate mode, 2000x1124 physical pixels, 60 Hz monitor, dev profile: after the log reported fully loaded and while draw count remained 2,136, 28 one-second summaries were near 30 FPS with acquire near 29 ms. The subsequent summaries had zero draws and near-500 loop FPS; they are not rendering-throughput measurements. This was a single run with foreground and occlusion state not independently controlled.
- Metal System Trace recorded command-buffer events for the offline viewer in `.scratch/baseline-metal.trace`, but the initial trace was affected by build contention and startup. A second trace did not contain sustained scene rendering and was stopped during a prolonged save. Neither establishes a native GPU-duration comparison.
- wgpu Metal surface acquisition explicitly returns Occluded for a hidden window; this is a plausible explanation for zero draws, not a confirmed observation of that return value.
- The existing renderer uses Discard, upload staging reuse, and wgpu-hal Metal includes autorelease pools. The fixed-count GPU-culling experiment remains opt-in.

## Current changes and limitations

- The working tree adds sample frame identity, once-only aggregation, stale/invalid sample suppression, mapping failure release and endpoint routing; these are incomplete protection, not proof of correct Metal timings. Positive timestamp pairs can still mix old and new writes. The production GPU timing defect is not solved and must not be used to establish a GPU bottleneck.
- Formatting, workspace Clippy, and workspace tests passed before the latest stale-display expiry change. Final macOS workspace Clippy and tests passed after that change: 5,969 passed, 32 ignored; the PowerShell check is unavailable here. The user has not tested this build, and no performance fix or target acceptance has been established.
- Next: establish a reliable Metal timing mechanism independently, reproduce the stale-positive case in isolation and compare against a valid native trace; then profile CPU scene generation, GPU stages and presentation in matched foreground runs. Optimize the measured bottleneck and validate with the human on the real scene before a PR. Windows and Linux regressions, visual scenarios, timing overhead and long-duration tests remain open.
