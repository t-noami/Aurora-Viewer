# Metal performance repair plan

Date: 30 October 2026. Status: plan documented; implementation and performance acceptance remain open.

## Revision: official v0.8.3 baseline and launch safety

This revision supersedes the old fork baseline and any instruction to continue performance work directly on it. The user reports no visible graphics improvement in v0.8.3. Treat that as an unresolved user observation, not proof of a particular renderer defect.

Baseline: official `v0.8.3`, commit `7030025c2678c9b428aac1b53d385b752c326a55`. Use a new dedicated `codex/` branch/worktree from that exact tag. Preserve the previous repair worktree and its uncommitted changes; inventory its diff and untracked files, then selectively port only still-needed changes. Do not reset, overwrite, blindly merge the old fork, or switch the dirty worktree in place. Do not silently substitute current main or a beta release.

Read [the offline launch incident and prevention rules](OFFLINE_DEMO_LAUNCH_SAFETY.md) before any executable or UI operation. Use the official `AURORA_DEMO=1 cargo run -p aurora-viewer -- --title "Test metal-v083"` launch procedure; a custom launcher or startup redesign is not a prerequisite. No normal-mode launch, app activation by path/name, or login-screen inspection is authorized for this task.

### Ordered implementation and evidence

1. Preserve the old work; record the new base SHA and dependency versions. Compare the upstream renderer with the old timer patch and minimal repro. Re-evaluate stale samples, timestamp ordering, readback ownership and actual final pass against v0.8.3; do not assume the old diagnosis or patch applies unchanged.
2. Launch using the official procedure: `AURORA_DEMO=1 cargo run -p aurora-viewer -- --title "Test metal-v083"`. Apply profiling/capture variables to that same command when needed. Retain its process/log identity and verify the demo scene using its own capture. Do not implement a custom launcher, stub tests or startup redesign as prerequisites. The incident was the separate launch-capable UI inspection, not evidence that the official demo environment switch is defective. Never open a second normal instance to inspect the demo. Run `./scripts/check.ps1` as required; report platform-specific execution failures accurately.
3. Establish a new verified v0.8.3 baseline. Retain exact binary identity, launch manifest, owned PID/lifetime, positive demo-mode log marker and capture from that same process. Old `.scratch/v083-standard.log` and `.png` are quarantined historical artifacts: the session also opened a normal login window and the user disputes the demo observation. The previously quoted 65.2 FPS is not an accepted baseline or evidence of improvement. Do not reopen or redistribute the normal-login capture.
4. Measure standard and 600-object demos, then water/transparency/shadows/AA/HUD/selection cases. Use at least three matched runs, each with a declared loading/warm-up exclusion and >=60 seconds of stable visible rendering. Record physical resolution, camera, quality, display rate, present mode, power/thermal conditions, draw/object counts, scene-update rate, CPU encode/submit, drawable acquire and native GPU timings. Exclude occluded/zero-draw intervals and capture overhead. Report aggregate FPS as total frames / measured time; report frame-time median/p95 from frame samples, not a percentile of per-second averages. If only summaries exist, label their aggregation explicitly.
5. Make the visual baseline separately reviewable. Keep quality and features fixed; use matching synthetic scenes and camera captures to check lighting, materials, textures, geometry, water, transparency, shadows and AA. Disabling shadows, reducing resolution or quality is a diagnostic experiment, not an accepted performance fix. Specific real-scene visual defects remain pending human evidence; do not invent them from the broad report.
6. Attribute the bottleneck using CPU/GPU/presentation evidence. Native Metal/Instruments observation is required before treating GPU timestamp values as authoritative. Missing tooling must not cause invented timing or stall all independent CPU, surface and visual analysis. GPU=0 means unavailable until proven otherwise. A 60 Hz display or long drawable wait does not by itself prove either GPU saturation or a hard render-throughput limit.
7. Implement the smallest measured improvement using sections 3–5 below: target lifetime/load/store and pass boundaries, draw submission/batching, uploads, or surface scheduling according to evidence. Preserve upstream native Metal integration and cross-platform behaviour. Run matched before/after experiments for each change and reject regressions. Diagnostic correctness alone does not complete this task.
8. Deliver code, required checks, demo captures, timing tables, native trace interpretation and unresolved limitations. Human real-scene validation remains mandatory; only the human initiates normal mode and logs in. The existing minimum 120 FPS comparable-real-scene acceptance remains open until demonstrated without sacrificing graphics quality.

### Source and artifact map

- `crates/aurora-render/src/renderer.rs`: timestamp lifecycle, render-pass graph, attachment lifetime.
- `crates/aurora-render/src/main_thread.rs`, `types.rs`: CPU draw preparation, backend/stat propagation.
- `crates/aurora-viewer/src/frame_profile.rs`: interval aggregation and diagnostic semantics.
- `crates/aurora-render/src/upload.rs`: transfer/allocation reuse; verify current implementation before changing.
- `ARCHITECTURE.md`: native Metal integration and thread ownership; `TASKS.md`: macOS work and performance still open in the v0.8.3 snapshot.
- `docs/TESTING.md`, `docs/MACOS.md`, `.github/workflows/macos.yml`: supported demo controls, packaging and platform validation.
- Previous repair worktree (read-only reference): `metal-performance-repair/Aurora Viewer`; preserve its timer changes and minimal repro, but revalidate against this tag.
- Official release: https://github.com/Aurora-Viewer/Aurora-Viewer/releases/tag/v0.8.3 . Previously downloaded arm64 DMG checksum: `9ac81ad821539002b1123955f27e9668b2552f2875b6f38729a7bf839b0dacb5` (verify again before reuse).

## Objective and scope

Keep the shared Rust/wgpu renderer and preserve Windows/Linux behaviour, graphics quality, protocol and existing user settings. Identify and repair the causes of the reported approximately 10 FPS on macOS versus 120–240 FPS on Windows in the same location. Acceptance requires at least 120 FPS in a comparable real scene, aiming for the reported 120–240 FPS range. An offline demo at 30–60 FPS is not acceptance. Record hardware differences; the target is not a prediction that different hardware will achieve identical throughput.

Only the human connects to a grid and performs real-scene validation. Do not access credentials. Use offline scenarios for automated development. This plan supersedes the earlier fork-based port planning as the implementation baseline; it does not mark outstanding port or performance work complete.

## Evidence and unresolved questions

The performance investigation recorded stale GPU samples and timestamp pairs whose end precedes their start. Previously reported GPU medians (including 15.84 and 24.15 ms) must not establish a GPU bottleneck. The real-scene cause remains unconfirmed.

The inspected performance worktree routes timestamps through multiple command encoders, finishes encoders in parallel, and selects the ending timestamp according to HUD, selection and SMAA conditions. Query resolution occurs before the final egui rendering. These observations motivate an audit; they do not establish that parallel encoding or a particular branch causes the invalid samples.

The default CPU draw-list path measured about 60 FPS and the experimental fixed-count GPU-culling path about 30 FPS in one matched offline comparison. Presentation pacing limits interpretation. Keep the experiment opt-in until representative performance and visual checks justify it.

The existing renderer already uses Discard in some passes and reuses upload buffers. The installed wgpu-hal 30.0.1 Metal backend already contains autorelease pools. Its buffer storage selection uses mapping usages to select Shared versus Private; removing COPY_DST alone is not a general storage-mode optimization. Its texture implementation selects Memoryless for transient usage when the device supports it. Audit exact version behaviour rather than copying generic examples.

## Design decisions from the research

| Topic | Decision |
|---|---|
| Load/Store | Use Clear only when previous contents are unnecessary; use Discard only after the final consumer. Do not replace compositing Load operations indiscriminately. |
| Transient targets | Consider only attachments with no cross-pass preservation, sampling or copy requirement, subject to actual format and feature support. Discard alone is not proof of memoryless allocation. |
| Pass merging | Merge only compatible passes without intervening resource dependencies. Preserve water refraction, depth consumers and ordering. |
| MSAA | Preserve existing AA options. Do not force 4x or assume MSAA is free. |
| Presentation | Preserve VSync preferences and capability-based fallback. Do not force FIFO or confuse display refresh with maximum render throughput. |
| UMA transfers | Prefer improvements to the shared upload path. Direct mapping is an optional measured optimization, not an assumption of zero cost. |
| Autorelease pools | Audit render and helper threads plus existing HAL coverage; add scopes only where ownership and lifetime evidence justify them. |
| Backend selection | Keep existing platform policy. Changing to PRIMARY is not by itself a performance fix and must not silently change the Windows backend. |
| Shader layout | Follow WGSL alignment and size rules; bytemuck alone does not prove shader-layout compatibility. Preserve explicit padding and verify relevant layouts. |
| Scope | Preserve supported desktop platforms. Mobile/Web support and a separate native Metal renderer are not prerequisites for this repair. |

## 1. Repair and independently validate GPU timing

Primary areas: `crates/aurora-render/src/renderer.rs`, `crates/aurora-viewer/src/frame_profile.rs`, and performance UI consumers.

- Associate frame identifiers, query ranges and readback slots. Aggregate each completed sample once.
- Require matching begin/end samples from the same frame. Distinguish unavailable, invalid and stale data from zero workload.
- Audit query initialization, writes, resolve, submission, readback and reuse across encoders and frames in flight. Check actual wgpu Metal lowering and query reset behaviour.
- Build a minimal comparison of single versus multiple encoders and serial versus parallel finishing. Treat serialized execution as a diagnostic experiment, not a default production fix.
- Exercise ending-timestamp routes with HUDs, selection and SMAA on/off. Define scene timing separately from a UI-inclusive GPU frame interval.
- Compare with Xcode GPU capture and Instruments Metal System Trace. Check tool availability first; document any unavailable independent measurement.
- Measure timing enabled versus disabled. Never add a blocking GPU wait to the normal frame loop to make diagnostics easier.

Exit: several minutes of progressing samples; invalid/old samples excluded; tested routing cases; no unexplained mismatch with native observations; measurement overhead recorded. If the defect reproduces in a minimal upstream case, isolate that case and choose a narrow workaround or dependency fix based on evidence.

## 2. Establish a reproducible CPU/GPU/presentation baseline

Record scene-update, draw-list generation, command encoding/finish/submit, GPU stages, surface acquisition and presentation separately. Where stages overlap, do not sum them as serial frame costs. Record scene generation identifiers so presentation of repeated content is distinguishable from new scene production.

Use the standard demo, the 600-object streaming scene, a draw-heavy scene and scenes containing water, transparency and shadows. Keep camera, physical resolution, graphics settings, AA, loading state, foreground visibility and object counts fixed. Record backend, device features, hardware, OS/toolchain, build revision/profile, refresh rate, selected present mode, transfer bytes, draw counts and memory observations.

Use the default development profile for routine checks; use release only for deliberate performance comparisons. Compare repeated matched runs and retain frame-time median and p95, sample counts, captures and diagnostic logs in the owning worktree's ignored `.scratch/` directory. Exclude warm-up explicitly. If presentation caps the result, use an offscreen render-throughput test as supplementary evidence, not as real-scene acceptance.

Exit: evidence identifies the dominant interval and controlled experiments distinguish its leading causes. Missing Windows hardware/data must be stated, not replaced with an assumed equivalent workload.

## 3. Reduce unnecessary TBDR bandwidth and pass boundaries

Map each render target's producer, consumers, last use, format, sample count, usages, load/store operations and resolve/copy operations before editing it.

- Remove unnecessary loads, stores, intermediate copies and resolves.
- Apply transient attachments only where the complete lifetime permits it.
- Merge compatible passes while preserving resource hazards and visible behaviour.
- Measure the depth prepass on Metal; retain consumers such as occlusion and depth-dependent effects unless an equivalent replacement is validated.
- Preserve Aurora's reversed depth convention. Do not import example Clear(1.0)/Less settings into its Clear(0.0) path.
- Measure each change independently before combining it with other optimizations.

Exit: reproducible reduction in relevant pass time or bandwidth cost, with no regression in water, transparency, shadows, AA, selection, HUDs, postprocessing or capture. Recheck Windows/Linux instead of assuming a tile-GPU optimization is universally faster.

## 4. Optimize the measured bottleneck

### CPU command generation and draw submission

If this dominates, reduce state changes and native draw calls through batching and suitable instancing. Distinguish GPU visibility calculation from command submission savings: fixed-count indirect drawing may retain many native calls. Investigate Metal Indirect Command Buffers only after determining what wgpu exposes and whether backend-specific integration can preserve synchronization, resource ownership and maintainability. Keep the working fallback.

### Uploads and dynamic data

If transfers or allocation dominate, inspect `crates/aurora-render/src/upload.rs`; reuse existing mechanisms before replacing them with StagingBelt. Aggregate small updates, remove redundant uploads and temporary allocations, and measure bytes and time. Use write_buffer_with where it avoids an actual intermediate allocation. Consider mappable primary buffers only behind feature checks and measured benefit; rotate completed buffers, unmap before GPU use and never overwrite in-flight data.

### Memory lifetime

If resident memory grows, trace render/helper-thread Objective-C lifetimes and existing autorelease-pool coverage. Separate caches, allocations awaiting GPU completion and leaked objects. Add justified pool boundaries and repeat a long-running load/unload test; do not equate an advisory working-set threshold with free memory.

Exit: matched-run improvement survives repetition and long-session testing, without frame-loop waits, unbounded memory growth or cross-platform regressions.

## 5. Validate surface lifecycle and presentation

Audit `main_thread.rs`, `render_thread.rs`, `present.rs` and the viewer window-event path. Verify physical-size changes and scale-factor changes, display migration, minimization/restore, occlusion, zero-size windows and surface recovery. Avoid needless reconfiguration, and record actual selected mode and dimensions. Do not change colour space, backend or VSync defaults merely to improve a benchmark.

Exit: correct rendering and stable recovery across those transitions, with timing data that separates acquisition/presentation pacing from CPU and GPU execution.

## Delivery and acceptance gates

Implement in separate reviewable units: timing correctness; baseline/native instrumentation; targeted pass/target improvements; measured submission/upload improvements; lifecycle fixes where required. Prioritize sections 3 and 4 using section 2 results rather than committing to a wholesale rewrite.

For each viewer-changing unit: run appropriate workspace tests, formatting, Clippy and the required project checks; run affected and neighbouring demo scenarios; inspect before/after captures. Record unavailable checks explicitly. Changes to diagnostic variables must update TESTING.md; new modules must update ARCHITECTURE.md.

Final human validation uses comparable real-scene camera, physical resolution, settings and loading state, recording machine differences. At least 120 FPS is required; a demo-only improvement or 60 FPS does not close the task. If a hardware or presentation limit prevents the target, supply measured evidence and leave acceptance open for the human's decision. Follow the repository's human-validation, PR, review and merge gates.

## Sources and interpretation

The supplied research was reviewed as design input; its examples are not a version-matched patch. In particular, universal Clear/FIFO/4x-MSAA rules, free-MSAA claims, and storage-placement guarantees from removing COPY_DST are not adopted. No external discussion or private conversation needs to be published with this document.

- [wgpu 30.0.0 Buffer: mapping, staging and CPU/GPU exclusivity](https://docs.rs/wgpu/30.0.0/wgpu/struct.Buffer.html).
- [Apple: Explore Live GPU Profiling with Metal Counters](https://developer.apple.com/videos/play/tech-talks/10001/): stage-boundary sampling, sample validity and CPU/GPU timelines.
- [wgpu-hal 30.0.1 Metal device implementation](https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/metal/device.rs): inspect storage modes and transient support against the dependency actually used.
- [wgpu-hal 30.0.1 Metal command implementation](https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/metal/command.rs): inspect timestamp lowering and autorelease scopes.
- [wgpu-hal 30.0.1 Metal surface implementation](https://github.com/gfx-rs/wgpu/blob/v30.0.1/wgpu-hal/src/metal/surface.rs): inspect drawable acquisition and presentation.
