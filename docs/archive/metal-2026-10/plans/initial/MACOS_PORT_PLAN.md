# macOS port plan

Status: proposal, 28 October 2026. Based on `origin/main` at `b6fbd94c`. The initial macOS workspace check and a native Metal device probe have been performed; the workspace does not yet compile and no wgpu runtime or viewer validation has been performed. Implementation decisions and support claims depend on the results below.

## Goal and scope

Produce a macOS viewer with the same visible behaviour and protocol as the existing viewer, without regressing Windows or Linux. Start feasibility testing on Apple silicon; decide the minimum macOS version and whether to support Intel Macs after measuring actual hardware, dependencies and distribution constraints. The first milestone is a working local demo, not a claim of feature parity or release readiness. Human validation is required before a viewer-changing PR; only the human connects to a grid or handles credentials.

## Rendering options

| Route | Assessment | Decision |
|---|---|---|
| wgpu Metal backend | wgpu already sits between Aurora and the GPU and provides a native Metal backend on macOS. WGSL remains the shared shader input. Vulkan-specific code and feature assumptions still need work. | First feasibility candidate; not yet proven compatible or faster. |
| wgpu Vulkan via MoltenVK | Retains the Vulkan API path but maps a Vulkan subset to Metal. wgpu requires its `vulkan-portability` feature on Apple platforms; runtime integration, portability and the existing direct Vulkan code must be checked. | Comparison or fallback if Metal proves unsuitable; no assumption of feature or performance equivalence. |
| Direct Metal renderer | Uses Apple's graphics API directly, but entails a separate renderer, shader and resource management path, and ongoing parity work. | Consider only if a measured, material limitation cannot be addressed through wgpu. |
| wgpu GLES via ANGLE | wgpu documents ANGLE as an optional macOS backend. Compatibility with Aurora's compute, binding arrays and indirect rendering is unproven and may require substantial redesign. | Research fallback only; probe requirements before considering it. |

macOS does not have a native Vulkan driver according to the MoltenVK project. Both the native wgpu Metal route and direct Metal avoid MoltenVK; neither removes the need to test hardware capabilities. Benchmark comparable scenes and settings before selecting a backend: no performance ranking is established here.

Sources checked 28 October 2026: [wgpu backend and shader features](https://wgpu.rs/doc/wgpu/documentation/features/index.html), [wgpu 30.0.1 feature capabilities](https://docs.rs/wgpu/30.0.1/wgpu/struct.Features.html), [MoltenVK design and limitations](https://github.com/KhronosGroup/MoltenVK), [Apple Metal documentation](https://developer.apple.com/documentation/metal).

## Known starting points

- `crates/aurora-render/src/main_thread.rs` selects and enumerates Vulkan explicitly. `open_device` requires texture binding arrays, non-uniform indexing, at least 64 array elements, and `INDIRECT_FIRST_INSTANCE`; support must be measured on each candidate backend and GPU.
- `dedicated.rs`, `vram.rs` and `main_thread.rs` access Vulkan HAL or use `ash`. Audit compilation and behaviour on Metal, with safe alternatives for allocation and memory accounting. Check indirect draw fallbacks, texture compression, depth clipping, timestamps and visual equivalence; some are already optional, not necessarily equivalent in practice.
- `crates/aurora-updater/src/media_pack.rs` treats every non-Linux target as Windows and selects Windows binaries. Prevent an incorrect download and audit plugin paths, shared memory, process launch and media keys before enabling media on macOS. A working macOS media pack is not established.
- The release and media workflows target Windows and Linux; updater installation has Windows-only paths. The existing macOS task and font lists are unvalidated. Inventory platform-specific code and native dependencies before estimating completion.

## Feasibility findings (28 October 2026)

On the investigated Mac, macOS 15.7.7 reports an Apple M2 Pro GPU. A native Metal probe found Apple7 and Mac2 families and Argument Buffers Tier 2; its recommended maximum working set was 22,906,503,168 bytes. These are Metal device properties, not an Aurora or wgpu runtime test.

The wgpu 30.0.1 Metal adapter source enables texture binding arrays and non-uniform indexing when MSL 3.0, texture arrays and Tier 2 argument buffers are available. Tier 2 also gives a binding-array limit of 1,000,000 in that source. The measured Tier 2 is encouraging, but the full conditions, actual wgpu-reported features and limits, device creation and shaders must be checked on this machine. `INDIRECT_FIRST_INSTANCE` is documented on Metal Apple3+ or Mac1+; the device reports Apple7 and Mac2. Do not treat these inferences as a working viewer.

A concrete performance limitation needs its own milestone: wgpu 30.0.1 documents `MULTI_DRAW_INDIRECT_COUNT` for Vulkan and DX12, not Metal. Aurora's `GpuCull::supported` selects its CPU-built draw lists when that feature is absent (`main_thread.rs`); that fallback is a first bring-up route, not evidence of equivalent performance or visuals. Measure it on representative scenes. Investigate a GPU-driven alternative, including whether it can be expressed with wgpu or requires backend-specific integration, without assuming either approach is feasible.

`dedicated.rs` already falls back to wgpu allocation when its Vulkan HAL access is unavailable. This is a runtime fallback, not proof that the Vulkan-specific references compile on a Metal-only build or that allocation and memory-pressure behaviour is acceptable. `vram.rs` and adapter VRAM detection also rely on Vulkan queries; assess a Metal-aware budget or a documented safe fallback for unified memory.

A baseline `cargo check --workspace --locked` was run on the plan's `b6fbd94c` worktree with Xcode 16.2 and Rust 1.99.1 (`aarch64-apple-darwin`). Dependencies downloaded and compiled, but `aurora-render` failed with six errors: `wgpu::hal::api::Vulkan` and `wgpu::hal::vulkan` are configured out for the default macOS Metal build. The references are in `dedicated.rs`, `vram.rs` and `main_thread.rs`. The Vulkan code must therefore be conditionally compiled, with working non-Vulkan implementations, before the workspace can compile. The check stopped there, so further errors may remain. No viewer or Metal rendering was run.

Sources: [wgpu 30.0.1 Metal adapter](https://raw.githubusercontent.com/gfx-rs/wgpu/v30.0.1/wgpu-hal/src/metal/adapter.rs), [wgpu 30.0.1 features](https://docs.rs/wgpu/30.0.1/wgpu/struct.Features.html), [wgpu backend features](https://wgpu.rs/doc/wgpu/documentation/features/index.html).

## Milestones and exit criteria

1. **Baseline and inventory.** Create an isolated task worktree based on current `origin/main`. Record macOS version, machine and GPU, Xcode tools, Rust toolchain and dependencies. Run the workspace build/check, classify compile and link failures, and inventory OS-specific paths. Do not modify another task's work. Consult the Legacy Viewer sources for relevant platform behaviour. Exit: reproducible build blockers and a prioritized inventory.
2. **GPU feasibility.** In the task worktree, enable the Metal backend and collect adapter features, limits and formats; check the exact requirements above and all direct Vulkan access. Run a minimal device/surface and shader test, then a representative demo. Record whether CPU draw-list fallback is selected, then separately investigate and measure a GPU-driven Metal alternative. If a required capability is absent, prototype a viable alternative with no visible regression and quantify its cost. Compare MoltenVK on the same machine and scenarios if it is a realistic candidate; document feature flags, loader/runtime packaging and limitations. Exit: measured compatibility, screenshots, validation errors, frame-time and memory results, and an evidence-based backend decision or a documented blocker.
3. **Working demo.** Fix compile/runtime portability, window lifecycle and resource paths. Exercise resizing, Retina scaling, multiple displays, keyboard shortcuts, Japanese IME, clipboard, file dialogs, fonts, audio output and clean shutdown. Capture screenshots of representative scenes and compare with the existing viewer and the Legacy Viewer under matching settings. Exit: repeatable demo scenarios with no known visual regressions; list anything deferred.
4. **Platform integrations.** Test Keychain-backed saved credentials without the agent accessing them, OS notifications, URL handling, permissions and microphone/audio devices. Investigate media plugins and voice independently, including build, architecture, licensing, bundle layout and runtime permissions; explicitly disable unavailable paths with a clear user-facing explanation. Review updates and data/cache paths, migration, sleep/wake and device changes. Grid-dependent tests remain for the human.
5. **Distribution and automation.** Add macOS CI for formatting, Clippy and workspace tests as feasible, plus actual GPU/demo validation on supported hardware. Design an `.app` bundle with correct resources, architecture, version and launch behaviour; decide archive/installer, signing, notarization and update policy before publication. Verify fresh install, upgrade, permissions and launch on supported macOS versions. Update build, testing and user documentation, including the generated wiki table when adding guides. Preserve Windows and Linux CI and tests.
6. **Release gate.** Run the project's required checks and relevant demo regressions; compare screenshots and profile representative scenes. Ask the human to validate the build and, where needed, log in and test networking, teleport, voice and media themselves. Record validated versions, hardware, results and remaining limitations; follow the repository's PR, review and merge rules. A release happens only when requested by the human.

## Decisions and open questions

- Which macOS versions and Intel GPUs are supported? Decide from capability probes, dependency support and available test hardware, not assumptions.
- Which media and voice capabilities are required for the first public build? Identify missing dependencies and agree on any visible limitations before release.
- How will signing, notarization, distribution and updates be provided? Verify the available Apple development setup and publication requirements before selecting a policy.
- Record exact test hardware, backend, feature/limit report, demo inputs, images and performance measurements. A successful build alone is not runtime or grid validation.
