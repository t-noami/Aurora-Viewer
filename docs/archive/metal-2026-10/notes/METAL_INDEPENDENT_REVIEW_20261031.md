# Metal investigation: parent verification

Date: 31 October 2026 (JST). Read-only investigation; no new measurement, viewer launch, build or code modification.

## Directly verified evidence

- Current repair diff (`metal-v083-repair`) changes `main_thread.rs` texture page capacity and adds a unit test. It contains no draw scheduling, ICB, render-pass or shader performance change. The old renderer timing patch is not part of this current source diff.
- Baseline is official v0.8.3, 7030025c. Cargo.lock pins wgpu / wgpu-hal 30.0.1. `GpuCull::supported` requires MULTI_DRAW_INDIRECT_COUNT, and this Metal path uses CPU draw lists.
- Aurora `renderer.rs:5951–5964` increments its `calls` counter once per grouped multi-draw API call. Installed wgpu-hal 30.0.1 `metal/command.rs:1608–1629` loops through draw_count and records individual native indirect indexed draws. Therefore calls=13 does not mean 13 native Metal draws, and this path is not ICB execution.
- `renderer.rs:4987–5023` measures acquisition around get_current_texture; line 7695 stores this duration. Metal `surface.rs:347–380` reaches CAMetalLayer.nextDrawable. This is a wait location, not identification of the original limiting stage.
- Metal `surface.rs:261–265,336–337` maps Immediate to displaySyncEnabled(false). Existing logs select Immediate with vsync=false, screen_hz=60, fps_cap=120 and limiter=0.00. Calling this simply an enabled VSync/60 FPS cap is not supported.

## Recalculation of existing demo summaries

No new benchmark was run. Filter: discard the first 20 seconds after the first perf summary in each file and require draws > 0. Values below are medians of one-second summary values, NOT raw frame-time medians, aggregate FPS or fresh results. Capture intervals are not independently excluded by this exploratory calculation; use these only to verify the dominant pattern.

| Existing log | summaries retained | FPS summary median | frame-ms summary median | acquire-ms summary median | draws | calls |
|---|---:|---:|---:|---:|---:|---:|
| v083-standard-1.log | 68 | 60.0 | 16.67 | 15.325 | 197 | 13 |
| v083-stream-1.log | 60 | 60.0 | 16.67 | 15.400 | 2140 | 13 |
| v083-stream-fixed-2.log | 71 | 60.0 | 16.67 | 15.360 | 2140 | 13 |
| v083-stream-fixed-3.log | 26 | 60.0 | 16.67 | 15.450 | 2140 | 13 |
| v083-stream-fixed-4.log | 82 | 60.0 | 16.67 | 15.440 | 2140 | 13 |

Logs are in the repair worktree's .scratch. They use 2000x1124, FXAA, shadows disabled, water effects disabled, SSR disabled. The user's 4K live scene and quality settings are not represented by these logs. The small scene and much larger draw count both reaching 60 FPS with acquisition dominating is consistent with presentation pacing, but is not conclusive proof. GPU backlog can also delay drawable availability. gpu=0.00 is not evidence of idle GPU.

## Conclusions before independent report

1. CPU-side per-object draw issuance can be reduced using native Metal ICB; GPU still executes drawing commands. Current shared wgpu path does not automatically provide this design.
2. Native ICB is a candidate architectural improvement, not an established cure for the reported 4K 10 FPS. Need to distinguish CPU submission, GPU pixel/bandwidth work, and presentation; no new tests are authorized now.
3. Apple's TBDR guidance targets redundant attachment traffic, pass breaks and visibility handling as well as CPU overhead. Preserve pass dependencies and visual behaviour.
4. Existing executable is a dev build: workspace opt-level=1, dependencies opt-level=3; release uses thin LTO. It is not an entirely unoptimized debug build, but its performance cannot be equated with the official release without a matched build comparison.
5. The texture fix restores workload previously missing. Before/after FPS alone is not an equal-quality speed comparison.

Primary reference: https://developer.apple.com/videos/play/wwdc2020/10602/ (GPU-generated draw commands with ICB; tile memory, Load/Store, pass merging).

Independent report and review follow below once received.
