# macOS FPS investigation — 29 October 2026

The human reports approximately 10 FPS on macOS versus 120–240 FPS on Windows at the same location. Windows hardware, actual output resolutions, settings and real-scene profiling have not been supplied; the offline results below do not establish the cause of that report.

## Offline measurement

Mac: Apple M2 Pro, 16 GPU cores, Metal. Development build, visible `AURORA_DEMO_STREAM=600,600`, vertical sync off, Immediate present mode, FPS cap 500, 60 Hz monitor. Medians of one-second samples after the demo reported fully loaded; runs are sequential, not simultaneous. The visible screenshots were inspected.

| Physical output | Culling | Samples | FPS | Frame ms | Acquire ms | Finish ms | Draws |
|---|---|---:|---:|---:|---:|---:|---:|
| 2560×1440 | CPU (default) | 44 | 30.0 | 33.33 | 31.79 | 2.25 | 2,136 |
| 2000×1124 | CPU (default) | 12 | 60.0 | 16.67 | 16.38 | 0.52 | 2,136 |
| 2000×1124 | Metal GPU experiment | 24 | 30.05 | 33.29 | 30.57 | 5.54 | 2,136 |
| 2000×1124 | CPU, release build | 12 | 60.0 | 16.68 | 16.71 | 0.53 | 2,136 |
| 1600×1120 | CPU (default) | 10 | 60.0 | 16.67 | 17.38 | 0.51 | 1,917 |

The near-identical-aspect 2560×1440 and 2000×1124 runs have the same reported draws and settings. Smaller output improved FPS, but window/UI differences, timing and presentation behavior remain possible influences. The two CPU/GPU culling runs were also capped by effective presentation behavior near 60/30 FPS; the experiment was slower in this scene and must not be enabled as a presumed fix. Acquire is the CPU wait in `surface.get_current_texture`; it may reflect compositor pacing, unavailable drawables, GPU backlog or other factors, not GPU execution time by itself. `gpu=0` in these profiles is not proof of zero GPU work: per-element GPU readings were unavailable.

## Implementation observations

- On Metal the default path builds draw lists on the CPU; the GPU-driven count path needs `MULTI_DRAW_INDIRECT_COUNT` and is not available in this backend.
- wgpu 30's Metal implementation loops over individual indirect draw commands inside `draw_indexed_indirect`; its 13 high-level calls are not necessarily 13 native draw commands. This is a plausible scaling cost for complex scenes, not a proven explanation of the reported 10 FPS.
- `AURORA_WINDOW_SIZE` uses logical pixels. At this Mac's scale factor the 1280×720 demo rendered at 2560×1440 physical pixels. Match actual physical resolution and graphics settings when comparing machines.
- The measured executable uses Cargo's dev profile (`opt-level=1` for workspace code); the release build in the same 2000×1124 offline scene also measured 60 FPS (12 loaded samples). This display-paced result does not isolate the maximum throughput or establish the effect in a busier grid scene. Windows CPU/GPU differences may also be material.

## Evidence

- `.scratch/fps-metal-baseline.log`, `.scratch/fps-metal-ratio.log`, `.scratch/fps-metal-small.log`, `.scratch/fps-metal-gpu.log`, `.scratch/fps-metal-release.log`; corresponding PNGs alongside them.
- Profiling code: `crates/aurora-render/src/renderer.rs` (`get_current_texture`), `crates/aurora-viewer/src/frame_profile.rs` (per-element GPU aggregation); wgpu 30.0.1 Metal `command.rs` (`draw_indexed_indirect`).
- Real-session cause remains unconfirmed until comparable scene settings, physical output resolution, hardware and frame breakdown are available. No agent connected to the grid.
