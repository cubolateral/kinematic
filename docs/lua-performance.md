# Lua performance benchmarks

Run `cargo bench --bench lua` from the repository root. The runner uses the
release profile, three warmup operations and seven timed samples. CSV output
reports median, minimum and maximum microseconds per operation. No additional
benchmark dependency is required.

Cases cover construction of 1,000 objects, static updates, simultaneous native
2D/3D tracks, 1,000 Lua signals, a 2,000-value simulation state, Game of Life
playback and backward seek, compatible reload, and CPU raster drawing.
Playback cases include periodic backward seeks; the large-state cases advance
continuously to include growing checkpoint history. Selective checkpoint cases
exist only in the historical results; the callback API was removed in round 6.

These timings measure the scene API, not SDL input, GPU rendering or MP4 export.
Run on an idle machine and compare identical workloads and build profiles.
The 60 FPS frame budget is 16.67 ms for the complete application, including
rendering and editor work.

## First five measured rounds (historical)

Measured on October 12, 2026, with an AMD Ryzen 3 3200G (four cores), Rust 1.94.1, Lua 5.4 and the Cargo bench profile. Values below are median milliseconds per operation; raw CSV files in `benches/results` retain sample ranges. Small differences can reflect CPU frequency and system noise.

| Case | Baseline | Round 1 | Round 2 | Round 3 | Round 4 | Round 5 |
|---|---:|---:|---:|---:|---:|---:|
| load_1000_2d | 24.3380 | 24.4113 | 24.4709 | 24.7512 | 24.7960 | 24.7956 |
| static_1000 | 1.4525 | 1.4021 | 1.3896 | 1.4259 | 1.4040 | 1.3994 |
| tracks_1000_2d | 1.6040 | 1.5280 | 1.5493 | 1.5393 | 1.5450 | 1.5383 |
| tracks_1000_3d | 1.3170 | 1.3031 | 1.2784 | 1.2909 | 1.2547 | 1.3079 |
| signals_1000 | 7.0854 | 6.8256 | 7.0595 | 6.3623 | 6.4278 | 6.3697 |
| cache_full_step | 1.8353 | 0.9333 | 0.8262 | 0.7308 | 0.7579 | 0.7468 |
| cache_full_seek | 0.5269 | 0.5213 | 0.0089 | 0.0088 | 0.0087 | 0.0089 |
| cache_selective_step | 1.8740 | 0.9061 | 0.8505 | 0.7451 | 0.7705 | 0.7469 |
| cache_selective_seek | 0.5216 | 0.5180 | 0.0089 | 0.0090 | 0.0089 | 1.1696 |
| life_step | 0.4851 | 0.2400 | 0.2380 | 0.2294 | 0.2108 | 0.2152 |
| life_seek | 41.2769 | 19.3876 | 20.6275 | 20.0759 | 18.6479 | 19.1536 |
| life_reload | 1.1517 | 0.9246 | 0.9671 | 0.8722 | 0.8624 | 0.8742 |
| life_draw_raster | 0.3459 | 0.3091 | 0.2998 | 0.2763 | 0.2379 | 0.2296 |

1. Compare snapshot keys directly instead of allocating Debug strings during sorting. Life seek fell from 41.28 to 19.39 ms.
2. Share immutable snapshot tables with Arc. Large-state checkpoint seeks fell from 0.521 to 0.00894 ms; restored Lua tables remain independent.
3. Find object properties directly in existing metadata, dynamic properties and uniforms instead of enumerating every property on each access. The 1,000-signal case fell from 7.06 to 6.36 ms.
4. Reserve Lua table array/hash capacity during snapshot reconstruction. Life seek fell from 20.08 to 18.65 ms and raster draw from 0.276 to 0.238 ms. An entity-indexed signal override candidate stayed near 6.4 ms and was reverted.
5. Add optional checkpoint/restore callbacks and share immutable checkpoint states when cloning simulation history. Full and selective continuous updates both took about 0.747 ms. Selective seek took 1.17 ms versus 0.009 ms for full snapshots because it rebuilt 2,000 cache values. Selective callbacks became effective in round 5; earlier selective cases used the original full-state behavior.

The selective checkpoint experiment was reverted at the user's request in round 6. Full immutable snapshots and shared checkpoint history remain. No RSS or peak-memory benchmark was performed.

Assessment after round 5: **7/10 for the tested scene workloads**, an engineering assessment against a 60 FPS frame budget, not a standardized score or measured application FPS. Single-frame animation updates remain under 1.6 ms, 1,000 signals consume 6.37 ms, and Life seek still takes 19.15 ms, exceeding one frame budget. GPU rendering, the editor and export need separate measurement. Full-property transactional snapshots and Lua/Rust state conversion remain significant costs.

Validation after the first five rounds: `cargo fmt --check` and `cargo check --workspace` passed. The first
`cargo test --workspace` run passed all 305 tests with four ignored. After adding
a seek-after-reload assertion, that regression passed, but the external LuaLS
initialization test timed out in full-suite runs and passed when run alone.
The final workspace run excluding that test passed 304 tests with four ignored.
No LuaLS implementation changes were made.


## Follow-up: panic, startup and development performance

The startup panic occurred when a saved nonzero timeline position triggered a
Lua simulation seek before `Editor::new_lua` had installed its documents.
Both native and Lua constructors now evaluate only after their state is ready.
The graphics regression opens Life with the timeline saved at 1.5 seconds.

The follow-up has five rounds, numbered 6–10 in the CSV filenames:

1. Remove selective checkpoint/restore callbacks, fix initialization order, and
   format snapshot diagnostic paths only on errors, preserving validation.
2. Save only animated properties and existing signal values for transactional
   rollback. Undo new signal writes on failure before restoring the previous
   frame. A regression covers an error after changing a previously untouched
   static property, another object's property, and an animated property.
3. Borrow simulation callback metadata instead of cloning it every step;
   traverse tables through `Table::for_each` and reserve frame/input capacity.
4. Reconstruct tables in bulk, using the sequence path for dense arrays and the
   general table path for sparse or mixed keys. An entity-indexed signal
   override candidate showed little benefit and was reverted after measurement.
5. Set development optimization to level 1, retaining debug assertions,
   overflow checks and debug symbols. Set the simulation example's configured
   frame rate from 30 to 60. Compilation can take longer and optimized locals
   may be less convenient to inspect in a debugger. Life evolves every six
   simulation steps, so increasing the project FPS also increases its generation
   rate; the counter uses `dt` and retains its speed in units per second.

Reproduce scene measurements:

```sh
cargo bench --bench lua
cargo bench --bench lua --profile dev
KINEMATIC_BENCH_FILTER=life cargo bench --bench lua --profile dev
CARGO_PROFILE_DEV_OPT_LEVEL=0 cargo bench --bench lua --profile dev
```

The last command measures development without optimizations. Debug results for
rounds 6 and 8 contain only Life cases. All other debug result files contain the
full suite. Round 9 includes the subsequently reverted signal-index candidate.

Reproduce the ignored graphics regression and frame benchmark:

```sh
SDL_VIDEODRIVER=offscreen cargo test --lib lua_simulation_startup_and_frames -- --ignored --nocapture
SDL_VIDEODRIVER=offscreen cargo test --release --lib lua_simulation_startup_and_frames -- --ignored --nocapture
```

This uses the Life source at 640×480 and 60 FPS, extends its duration to ten
seconds, opens a saved position at 1.5 seconds, and evaluates 180 consecutive
frame times. Each frame asserts that the scene reached the requested time.
There is a 17 ms wait outside the measured region so the editor's pacing cannot
skip evaluation. Timings include editor update, preview rendering, document UI,
ImGui submission, synchronized Skia submission and window swap. The wait is
excluded: these are frame processing costs, not a continuously measured FPS
counter. Startup includes SDL/OpenGL construction, editor creation and replay
until the preview is ready; it does not include launching the process or a cold
OS filesystem cache. The initial sandbox runs have no exposed hardware GPU; final comparisons also
run outside that sandbox to separate its effects. Hardware-driver and
display-presentation timings must be checked on the user's desktop.

### Development measurements

Median milliseconds; rounds 6–9 use opt-level 0, round 10 uses opt-level 1.
All values in this table were collected inside the sandbox. The first round-10
run outside it is retained separately as `lua-debug-round10-host.csv`.

| Case | Debug baseline | Round 6 | Round 7 | Round 8 | Round 9 | Round 10 |
|---|---:|---:|---:|---:|---:|---:|
| load_1000_2d | 259.5994 | — | 244.6813 | — | 240.9867 | 29.1676 |
| static_1000 | 17.5710 | — | 0.0498 | — | 0.0485 | 0.0029 |
| tracks_1000_2d | 19.5646 | — | 2.8233 | — | 2.7095 | 0.2009 |
| tracks_1000_3d | 16.7484 | — | 2.8219 | — | 2.7707 | 0.2040 |
| signals_1000 | 55.6034 | — | 38.8585 | — | 37.8690 | 5.8555 |
| cache_full_step | 3.5395 | — | 2.8653 | — | 1.8163 | 0.5155 |
| cache_full_seek | 0.1427 | — | 0.0259 | — | 0.0263 | 0.0019 |
| life_step | 0.7912 | 0.7104 | 0.6360 | 0.5742 | 0.4806 | 0.1591 |
| life_seek | 63.1016 | 55.1372 | 54.5420 | 49.3822 | 40.8374 | 14.2002 |
| life_reload | 4.5911 | 4.5098 | 4.4878 | 4.3483 | 4.3604 | 0.9872 |
| life_draw_raster | 1.3788 | 1.3893 | 1.3659 | 1.4133 | 1.0355 | 0.2455 |

### Editor measurements

The sandbox materially changes graphics timings. Comparing an old sandbox run
with an optimized run outside it exaggerated the improvement. The final
comparison reran the saved round-9 binary and both final binaries consecutively
**outside the sandbox**, with the same workload and no concurrent builds:

| Profile / revision | Startup to preview | Median frame | p95 | Worst of 180 |
|---|---:|---:|---:|---:|
| Unoptimized debug, round 9 | 450.62 ms | 16.01 ms | 18.44 ms | 19.99 ms |
| Development opt-level 1, round 10 | 112.89 ms | 4.95 ms | 6.25 ms | 6.50 ms |
| Release, round 10 | 98.74 ms | 4.54 ms | 5.94 ms | 6.22 ms |

Every measured steady-state frame in both final profiles fit the 16.67 ms budget.
The unoptimized development profile missed it at p95. Final scene evaluation
was 0.352 ms in development and 0.268 ms in release (medians). These timings do
not establish a universal minimum FPS for arbitrary scripts or desktop drivers.
The graphics measurements are offscreen, not a measurement of a physical
monitor's presentation rate.

The environment effect remains visible when comparing all profiles **inside**
the sandbox:

| Profile / revision | Startup to preview | Median frame | p95 | Worst of 180 |
|---|---:|---:|---:|---:|
| Unoptimized debug, round 8 | 754.20 ms | 28.34 ms | 37.91 ms | 41.91 ms |
| Unoptimized debug, round 9 | 725.57 ms | 27.16 ms | 36.42 ms | 41.54 ms |
| Development opt-level 1, round 10 | 391.41 ms | 19.08 ms | 22.96 ms | 24.94 ms |
| Release, round 10 | 372.38 ms | 18.69 ms | 22.76 ms | 24.58 ms |

That restricted graphics environment still misses the 60 FPS budget. Raw
`lua-editor-*-host.txt` files identify the final outside-sandbox comparisons.
`lua-editor-round10.txt` is the first outside-sandbox run (3.87 ms median);
`lua-editor-round10-sandbox.txt` records the inside-sandbox confirmation.

### Follow-up release measurements

Median milliseconds per operation.

| Case | Round 6 | Round 7 | Round 8 | Round 9 | Round 10 |
|---|---:|---:|---:|---:|---:|
| load_1000_2d | 25.1523 | 23.6116 | 23.2654 | 23.9001 | 22.9654 |
| static_1000 | 1.4309 | 0.0025 | 0.0025 | 0.0025 | 0.0024 |
| tracks_1000_2d | 1.5931 | 0.1599 | 0.1479 | 0.1562 | 0.1472 |
| tracks_1000_3d | 1.3931 | 0.1608 | 0.1497 | 0.1593 | 0.1489 |
| signals_1000 | 6.7075 | 4.6938 | 4.6744 | 4.6391 | 4.7218 |
| cache_full_step | 0.6018 | 0.5971 | 0.4451 | 0.4088 | 0.4093 |
| cache_full_seek | 0.0085 | 0.0013 | 0.0013 | 0.0013 | 0.0012 |
| life_step | 0.1773 | 0.1648 | 0.1509 | 0.1426 | 0.1417 |
| life_seek | 15.4825 | 14.7503 | 13.5167 | 12.8721 | 12.7755 |
| life_reload | 0.8335 | 0.8163 | 0.7929 | 0.8293 | 0.8110 |
| life_draw_raster | 0.2380 | 0.2295 | 0.2266 | 0.2144 | 0.2011 |


### Follow-up validation

`cargo fmt --check` and `cargo check --workspace` pass. Inside the sandbox,
`cargo test --workspace` timed out while initializing the external LuaLS server;
the isolated retry also timed out. Excluding only that test passed 303 tests,
with five ignored. The final complete `cargo test --workspace` run outside the
sandbox passed all 304 tests, with five ignored, including the LuaLS integration.
No LuaLS code was changed.

The ignored graphics startup regression passed in unoptimized development,
optimized development and release. Snapshot round trips, nested diagnostic
paths, transactional rollback, checkpoint seek and reload behavior pass.
`cargo run --bin kinematic -- --check examples/lua/simulation` validated both
example scenes at their new configured 60 FPS.

Assessment: **8.5/10 for the measured workload** against the 16.67 ms frame
budget. Both final profiles have substantial steady-state headroom outside the
sandbox, but startup, arbitrary user scripts, physical display presentation,
export performance and peak memory are separate limits. The score is an
engineering assessment, not a standardized benchmark rating.
