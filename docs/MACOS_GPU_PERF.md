# macOS GPU performance investigation (#3218)

Notes on the investigation of the low frame rate on Apple GPUs (issue
#3218), the changes made on the `perf/macos-gpu-perf` branch, how their
images were checked, and what is left. The quality bar of this work is
**60 FPS** in the measured scenes; it is not reached yet (see
[What is left](#what-is-left)).

## Background

- On the grid, in a busy store with the same settings, the Legacy Viewer
  keeps above 30 FPS with the GPU at about 80 %, where Aurora runs at 3–8
  FPS with the GPU at 100 %.
- #2169 (merged) changed the Mac defaults (logical resolution, the Legacy
  Viewer's Mac "Mid-High" values). It draws fewer pixels and features; the
  cost of each pixel is unchanged, and this issue is about that cost.
- A Metal System Trace on the grid (M2 Pro, world 2560 × 1387) gave, per
  frame:

  | Pass | GPU ms |
  |---|---|
  | SL mirror (`mirror`) | ~160 |
  | Blended faces (`scene b`, ~310 faces) | ~51 |
  | Opaque faces (`scene a`) | ~31 |
  | Local lights (64 → 0) | ~20 saved |

- **Root cause.** The Metal GPU counters show fragment occupancy at 12–15 %,
  ALU under 30 %, textures and bandwidth almost idle, and buffer stores in
  fragment shaders that write no buffer (register spills). The scene
  ubershaders (`object.wgsl`, `common.wgsl`) hold so many registers that
  the GPU mostly waits. Removing any one unused path alone gains nothing;
  removing all of them together gains about 5.5 ms: the peak register count
  decides. Hence the direction of the branch: compile each scene pipeline
  for what the frame really uses.

## Bench

All numbers come from the demo (`AURORA_DEMO=1`), never from the grid,
except where the grid is named.

- **Machine**: M2 Pro (16-core GPU), 5K display. Window 2400 × 1300 logical
  (4800 × 2600 physical); the world drawn at 2400 × 1300 ("Match the
  display") and upscaled to the window.
- **Settings**: the tester's grid Graphics settings, **weather off**
  (Aurora Weather costs about 35 ms on its own and is handled apart, see
  [What is left](#what-is-left)), the background FPS cap off (an unfocused
  window is otherwise capped at 30 FPS).
- **Scene**: `AURORA_DEMO_MIRROR=plaza` (a static mirror facing the plaza),
  `AURORA_DEMO_CROWD=40`, `AURORA_DEMO_LIGHTS=64`, and
  `AURORA_DEMO_VEILS=12,12` (12 blended curtains) for the blended-face
  load. This reproduces the grid's cost (about 6 FPS, GPU about 160 ms at
  the start).
- **Reading**: `AURORA_PROFILE=1`, the median of `gpu_frame=` (whole-frame
  GPU time, the only GPU figure Apple GPUs give) and of the frame rate over
  40 s. Per-pass costs come from a Metal System Trace, used only for the
  fragment intervals; overlapping passes are not attributed reliably, so
  each lead is confirmed by removing the pass (A/B).

### Progress

| Step | Scene | GPU ms | FPS |
|---|---|---|---|
| Start (upstream main) | 12 curtains | 88 | 10.8 |
| Branch head (21b3d17f8) | 12 curtains | 51 | 18.2 |
| Before the material classes | no curtain | 34.8 | 25.6 |
| Branch head (21b3d17f8) | no curtain | 33.9 | 27 |

Earlier steps, measured with the demo plaza (20 avatars) at the high
preset, 5K:

| Build | FPS | GPU ms |
|---|---|---|
| upstream main | 27.4 | 30.2 |
| + upscale and contact shadows (754cd8c5a, 6a67d208c) | 31.0 | 29.1 |
| + lean scene pipelines (c8e48cab9), low preset | 42.7 → 49.6 | |
| + lean scene pipelines (c8e48cab9), high preset | 30.6 → 34.3 | |
| static mirror without avatars (11f5ec3a3), 40 avatars | 16.1 → 20.6 | mirror 13.6 → 5.8 |

## Changes on the branch

| Commit | Change |
|---|---|
| 180cce2fa | The offline demo keeps its cache key in a file, not the system credential store (no access prompt at each rebuilt demo launch). `cache_key.rs`. |
| 301760b71 | `gpu_frame=` in the profile summary: the whole-frame GPU time. `frame_profile.rs`. |
| 754cd8c5a | The upscale to the window (CAS) writes its four kernels out instead of an indexed array, and reads the reduced scene through an sRGB view. Same image; 27.4 → 30.0 FPS at 5K. `rlva/resolution.rs`, `rlva_resolution.wgsl`. |
| 6a67d208c | Contact shadows sum their samples in four named lanes when the counts split into fours. Same math. `contact_shadows.rs`, `contact_shadows.wgsl`. |
| 2df23c35b | **Temporary** `AURORA_EXP_*` investigation switches; removed before any pull request. |
| c8e48cab9 | Lean scene pipelines (below). |
| 67122cda8 | The blended faces use the lean pipelines too. |
| 8bf927313 | Demo bench of a busy place (`AURORA_DEMO_MIRROR=plaza`) and measuring behind other windows (`AURORA_BACKGROUND=1`). |
| 11f5ec3a3 | Static mirror fix (below). |
| 21b3d17f8 | Material classes, sun shadow skip, single walk of the local lights (below). |

### Lean scene pipelines (c8e48cab9, 67122cda8)

`shaders/object.wgsl` takes an override constant `SCENE_VARIANT`:

| Value | Pipeline |
|---|---|
| 0 | full: every path |
| 1 / 2 | lean, classic sky / probe sky |
| 3 / 4 | lean, keeping the rez fade |

The lean variants compile out the portal clip, the debug views, the NaN
check, the water effects and the other sky's branch. `scene_variant.rs`
chooses the variant once per frame: full for a debug view, the NaN check,
water effects or a record with a portal clip; the rez variants while a
record rezzes, or while the rez grid is on and a texture loads. Opaque and
blended faces use the same choice.

### Static mirror (11f5ec3a3)

A static mirror leaves out the objects marked as not drawn in probes
(avatars and other dynamic objects), as the Legacy Viewer's cube snapshot
does with its dynamic render types. `FLAG_DYNAMIC` reaches the GPU cull
(`cull.wgsl`, `gpu_cull.rs`); a mirror set to dynamic still draws them. In
the bench the mirror pass went from 13.6 to 5.8 ms. This is a visible
change, on purpose: it matches the Legacy Viewer.

### Material classes (21b3d17f8)

A second override constant, `MATERIAL_CLASS`, specialises the scene
pipelines to the material of the faces they draw:

| Value | Faces |
|---|---|
| 0 | generic: fullbright, portal effect, or every face when `AURORA_SCENE_VARIANT=full` |
| 1 | legacy material (`FLAG_LEGACY_MAT`) |
| 2 | PBR |
| 3 | plain (neither) |

- In `object.wgsl`, `rec_pbr`, `rec_legacy_mat`, `rec_fullbright` and
  `rec_portal_effect` answer from the constant when it is set, and from the
  record's flags in the generic pipeline. They replace the flag tests of
  `eval_material_base`, `shade_material`, `legacy_may_glare` and the portal
  test of `fs_blend`.
- `scene_variant::material_class` gives a face's class;
  `DrawLists::blend_material` and `scene_material` (opaque, two-sided,
  mask, two-sided mask) are filled in `main_thread.rs` after the lists are
  copied. `second_view.rs` keeps them in step when it re-sorts the blended
  faces.
- `renderer.rs` draws each list in runs of the same class
  (`material_runs`), **in the list's order**, so the blend order is
  unchanged. Pipelines: `lean_blend_material` (4 variants × 3 classes) and
  `lean_material` (4 variants × 3 classes × 4 lists). The GPU-culled bins
  stay on the generic pipeline.

### Sun shadow skip (21b3d17f8)

In the classic lighting branch, the sun shadow is not sampled on a face
turned away from the sun (`dot(n, sun_dir) <= 0`), where the result does
not depend on it, unless the face has a legacy specular or environment
term, which uses it. The same skip is not possible in `shade_pbr`: its
probe specular occlusion uses the shadow.

### One walk of the local lights (21b3d17f8)

`ll_local_legacy` walks the local lights once and returns the diffuse and
the specular/environment sums together, where the classic legacy branch
walked them twice.

## Image checks

Every committed change was checked to draw the same image.

- Both builds (the base build and the new one) are capped with
  `AURORA_FPS_LIMIT=8`, so that a capture at the same frame
  (`AURORA_CAPTURE_FRAMES=600`) is at the same scene time: without the cap
  the faster build is further in its animations.
- Several captures per build; every pair is compared pixel by pixel
  (base vs base, new vs new, base vs new). A change is equal when base vs
  new stays within the noise of base vs base and new vs new (avatars,
  particles and water move between runs).
- A static region of the image (no avatar, no water, no particle) is also
  compared on its own, with a difference map.

Results:

| Change | Result |
|---|---|
| Lean pipelines, opaque | static regions within full vs full noise |
| Lean pipelines, blended | curtains identical; only moving things differ, as full vs full |
| Material classes, classic sky | within run noise |
| Material classes, probe sky | static regions differ by at most 1 level of 255, the same as new vs new |
| Upscale (754cd8c5a) | same image as the untouched build |

## Deferred G-buffer: not built

A deferred (G-buffer) prototype of the opaque faces was considered and not
built:

- The depth prepass already keeps the opaque overdraw near 1: each pixel
  is lit once, so a G-buffer would not reduce the lighting work.
- Its other gain, smaller shaders with fewer registers, is what the lean
  variants and the material classes already give, without a G-buffer's
  memory and bandwidth.

## Changes that alter the image: not taken

The large remaining costs come from places where Aurora draws more finely
than the Legacy Viewer. Matching the Legacy Viewer there would change the
image, and was **not taken**: the team is unlikely to accept a rendering
below Aurora's current one.

| Candidate | Upper bound measured |
|---|---|
| Blended faces lit by the 7 nearest local lights, as the Legacy Viewer's alpha shader, instead of all 64 | all local lights ~18 ms; blend shading ~16 ms |
| Fewer sun shadow PCF taps, as the Legacy Viewer (Aurora takes up to 7 × 7, twice in a cascade blend band) | shadow taps ~3.4 ms at the high preset |
| Lighter probe sampling for the faces with a legacy material | most of the opaque pass's remaining cost |

## What is left

### Weather

Aurora Weather in the demo costs about 35 ms per frame on its own. It is a
separate problem; every bench above runs with the weather off.

### 60 FPS not reached

60 FPS needs 16.6 ms. Without curtains the bench is at about 34 ms; from
the trace, per frame:

| Pass | ms |
|---|---|
| Opaque faces (`scene a`) | 12.6 |
| Blended faces (`scene b`) | 3.2 |
| Mirror | 2.9 |
| Impostors | 2.8 |
| Upscale | 2.4 |
| Water reflection | 1.4 |
| Shadow maps | 1.3 |

The opaque faces are 4160 plain, 170 with a legacy material and 80
fullbright; the probe sampling of the legacy-material faces is the costly
part. Skipping the probes, exposure, pick, glow, bloom, upscale or interface
alone saves 0–2 ms each.

Other leads, not yet followed:

- reflection probe captures as the Legacy Viewer paces them (rate,
  resolution, static objects only): about 6 ms with many textures;
- contact shadows on Apple GPUs (64-wide work groups on 32-wide SIMD);
- per-line shader costs from an Xcode GPU frame capture.

### Before a pull request

- Remove the `AURORA_EXP_*` switches (commit 2df23c35b).
- Give the surface `COPY_SRC` only while a capture is pending, so that it is
  framebuffer-only otherwise (about +0.8 FPS).
- ARCHITECTURE.md: `scene_variant.rs`.
- docs/TESTING.md: `gpu_frame=`, `scene_lean=`, `AURORA_SCENE_VARIANT`.
- TASKS.md line, with this issue.
- Check the pipeline compile and start-up time with the added lean and
  material pipelines.
- `check.ps1` (or its steps by hand on macOS).
- Human validation on the grid: frame rate in a busy place, blended faces
  (order, colours), legacy-material, PBR and fullbright faces, shadows on
  faces turned from the sun, places with many local lights, and the static
  mirror without avatars.
