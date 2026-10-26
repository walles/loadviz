# Linux Port Plan

Goal: a LoadViz tray icon (StatusNotifierItem) for Linux, written in Rust, that
can be built locally and run straight from the build output. Anything beyond
that goes into the TODO list in `linux/README.md`.

## Conventions

- Steps get checked off by prefixing their heading with "DONE". The commit
  that finishes a step also checks it off in this file.
- About one commit per step. Where a step says "test first", the failing tests
  go in their own commit, after verifying that they fail for the right reason
  (usually a `todo!()` stub), and the implementation goes in the next commit.
- Steps marked **Review** are committed but not pushed. Work stops there until
  Johan has reviewed them; they get pushed after that. All other steps are
  committed and pushed right away, to `johan/linux`.
- Both `libloadviz` and `linux/` stay clippy clean
  (`cargo clippy --all-targets --all-features` with zero warnings).
- The C API (`new_loadviz()`, `get_image()`) and the generated `loadviz.h`
  stay unchanged, so the macOS app should not need any changes. Step 7
  verifies that.

## DONE Step 0: Baseline

- Rust is installed using rustup, with the `aarch64-apple-darwin` target added.
- In `libloadviz`, verify that `cargo build --all-targets --all-features`,
  `cargo test --all-features` and `cargo clippy --all-targets --all-features`
  pass on Linux, and that `cargo check --target aarch64-apple-darwin` passes.
  Later steps must not break any of these.
- Save a copy of the generated `include/loadviz.h` so that Step 2 can diff
  against it.
- Commit: this plan file. Push, which creates `johan/linux` on GitHub.

## DONE Step 1: Read CPU load from `/proc/stat`

Replace the `todo!()` in `libloadviz/src/system_load.rs` with a Linux
implementation of `get_load_counters()`.

Test first:

- A platform independent test of `get_load_counters()` that runs on both
  macOS and Linux:
  - The number of CPUs equals `libc::sysconf(_SC_NPROCESSORS_ONLN)`. This
    catches counting the aggregate `cpu` line as a core.
  - No counter goes down between two calls.
  - The sum of all counters is above zero.
- Linux-only tests of a `parse_proc_stat(&str)` function, using canned
  `/proc/stat` text, verifying the column mapping:
  - user = `user + nice` (guest time is already included in `user`)
  - system = `system + irq + softirq + steal + iowait`
  - idle = `idle`

  The aggregate `cpu` line is skipped; only `cpuN` lines count.

Commit the tests failing on `todo!()`, then implement.

Verify: tests pass, and `cargo run --bin printload` prints plausible loads,
both while idle and while `cargo run --bin stressload` runs in another
terminal.

## DONE Step 2: Safe Rust API for `libloadviz`

So far the library is only used from Swift, through `extern "C"` functions
that pass raw pointers around. Make it usable from Rust without `unsafe`:

- `pub fn LoadViz::new() -> LoadViz` (and `impl Default`).
- Make `LoadViz::get_image()` public and have it return `&[u8]` (RGB, 3 bytes
  per pixel, `width * height * 3` bytes) instead of `*const u8`.
- Turn the `extern "C"` functions into thin wrappers around these.

Test first: commit `LoadViz::new()` and the new `get_image()` signature with
`todo!()` bodies, plus a test that checks the length of the returned image.
The commit compiles but the test fails on `todo!()`. The C wrappers panic in
that commit; the next one fixes that. Then implement.

Verify: Step 0 checks still pass, and `include/loadviz.h` is identical to the
copy saved in Step 0.

Also update the Output section of `libloadviz/README.md`, which describes
`getImage()` as the library's API, to mention the new Rust API.

## DONE Step 3: Tray icon showing one frame (Review)

Create a Rust crate in `linux/`:

- Binary name `loadviz`, `edition = "2021"`.
- Path dependency on `../libloadviz`, plus `ksni` with the `blocking`
  feature (no async code of our own).
- Commit `Cargo.lock`, ignore `target/`.

It should register a tray item and show one LoadViz frame as its
`icon_pixmap`. The icon is square, since tray hosts draw icons in a square
(GNOME's AppIndicator extension does, and so does KDE). Render exactly one
size per frame, never several.

The tray wants ARGB32 in network byte order while `libloadviz` produces RGB,
so add a conversion function. Test first: commit a `todo!()` stub plus tests
on a tiny image, then implement.

Verify: `cargo run` in `linux/` shows the icon in the GNOME top bar.

**Stop here**: Johan checks it visually, and we talk about the chosen icon
size before going on.

## DONE Step 4: Animate (Review)

Update the icon at 10 Hz, like the macOS app does: render a new frame,
convert it, and push it to the tray through the `ksni` handle, from a plain
thread that sleeps between frames.

Verify:

- The icon animates and reacts to `stressload`.
- Profile `loadviz` and `gnome-shell` together using
  `perf record -g -p <loadviz pid>,<gnome-shell pid>` (no sudo needed for our
  own processes), and check their CPU usage in `top`. Run `perf report` with
  `DEBUGINFOD_URLS=https://debuginfod.ubuntu.com` set, so it can fetch debug
  symbols for the GNOME libraries; fish doesn't pick up the system default.
  Look at:
  - How much of the `loadviz` time goes to the RGB to ARGB conversion. If it
    is noticeable, stop and discuss switching `libloadviz` to one shared pixel
    format for both platforms.
  - What `gnome-shell` spends its time on for our icon (D-Bus decoding,
    texture upload, scaling). If the icon size looks like the cost driver,
    bring it up in the icon size discussion. Otherwise, note findings in the
    README TODO list rather than fixing them here.

**Stop here** for Johan to check it visually.

## DONE Step 4b: Count iowait as idle

Added after Step 4. With iowait counted as system (Step 1), two cores showed
100% system load on an idle machine. The cause: Ghostty's event loop (libxev
on io_uring) keeps requests in flight, and io_uring marks tasks waiting for
completions as iowait, even when no disk is involved. Quitting Ghostty made the
load go away.

Change the mapping to:

- system = `system + irq + softirq + steal`
- idle = `idle + iowait`

Test first: change the expectations of the `parse_proc_stat()` test and commit
it failing, then change the mapping.

The cost: time spent waiting for swap-in reads from disk no longer shows. The
CPU side of swapping (page faults, `kswapd` and direct reclaim, zram/zswap
compression) is still system time. Step 6 documents this.

## DONE Step 4c: Guard macOS, screenshot and still rendering

Added after Step 4b. In a square icon the flame and cloud details look too
small, since the renderer sizes details relative to the image width
(`INTERNAL_DETAIL / width` and `DISTORTION_DETAIL / width` in
`renderer/flame.rs`, `10.0 / width` in `renderer/cloud.rs`), and the macOS icon
is twice as wide as it is high. Step 4d changes that for Linux. Before that,
add tests that pin down what the existing users get, so Step 4d can show that
they don't change:

- macOS icon: 2:1, for example 88x44 (2x Retina) and 44x22 (1x)
- `screenshot.webp` from `animated-webp`: 100x75
- the app icon source image from `still-webp`: 1024x1024. A smaller square
  (for example 128x128) is enough to catch a scaling change, and keeps the
  reference small.

Each test calls `Renderer::render_image()` with fixed loads (both user and
system load, so both flames and clouds show) and a fixed time, and compares the
result to a reference image committed next to the tests. The references are
generated by the current code, so these tests pass from the start; they guard
against regressions rather than drive a change.

Floating point results could differ slightly between x86_64 and aarch64, so
compare with a small per channel tolerance. Step 7 runs the tests on macOS,
which will show whether the tolerance is enough.

## Step 4d: Size details relative to the image height (Review)

Added together with Step 4c. The aspect ratio alone can't tell the square
Linux tray icon (needs larger details) from the square app icon source image
(must not change), so something has to say which one is being rendered. Make
the product default the natural rule for a status bar icon, and let the
offline tools ask for the old behavior:

- Default: size flame and cloud details relative to the image height, as
  `DETAIL / (2.0 * height)` where the code now says `DETAIL / width`. The
  macOS icon is exactly 2:1, so its output doesn't change. The Linux icon gets
  the macOS detail size with no new public API and no changes in `linux/`.
- Add a `Renderer` constructor for the old width based details, for example
  `Renderer::with_width_based_details()`. `animated-webp` (`screenshot.webp`)
  and `still-webp` (app icon source image) use it, so their output doesn't
  change. `Renderer::render_image()` is already only public for these tools
  and the benchmarks, so `LoadViz` and the C API don't change.
- The Step 4c screenshot and app icon tests render the way their tools do,
  using the new constructor. The macOS tests keep using the default. No
  reference image changes.

Test first: a test that, with the same load on every core, a 32x32 image
equals the left 32x32 pixels of a 64x32 image. That holds when details follow
the height, since all noise is sampled at `detail * pixel position`. It fails
with today's width based details. Commit it failing, then implement.

Verify: the Step 4c tests pass with unchanged reference images, `loadviz.h` is
identical to the Step 0 copy, and Johan checks the tray icon visually with
`LOADVIZ_FORCE_MAX=1`.

## Step 5: Quit menu item (Review)

Add a context menu with a single "Quit" item that exits the process. Help and
About go into the TODO list.

**Stop here** for Johan to check it visually.

## Step 6: `linux/README.md`

Write `linux/README.md` with:

- Prerequisites (rustup), how to build, and how to run from the build output
  (`cargo run --release`, or `target/release/loadviz`).
- A note that GNOME needs the AppIndicator extension (Ubuntu ships it enabled).
- An iowait section: iowait counts as idle, because io_uring marks tasks
  waiting for completions as iowait even when no disk is involved. Ghostty's
  event loop (libxev on io_uring) does that constantly, which showed up as
  two cores at 100% system load on an idle machine. The cost: time spent
  waiting for swap-in reads from disk no longer shows. The CPU side of
  swapping (page faults, `kswapd` and direct reclaim, zram/zswap
  compression) is still system time.
- A Profiling section, the Linux counterpart of the one in
  `macos/README.md`, based on what we did in Step 4: `perf record` on
  `loadviz` and `gnome-shell`, `DEBUGINFOD_URLS`, `LOADVIZ_FORCE_MAX`, and
  what the profile showed. For profiling the renderer in isolation, refer to
  `macos/README.md` and give the `perf` equivalent of its `sample` command.
- A TODO list containing at least:
  - Pause rendering when the icon isn't visible (screen locked etc.)
  - Install script (macOS has `install.sh`)
  - Autostart through `~/.config/autostart/*.desktop`. Like on macOS,
    starting enables autostart and Quit disables it. If we package as a
    Flatpak, use the Background portal instead, since a sandboxed app can't
    write that file.
  - Help and About menu items. On macOS, About shows the version and Git hash.
  - An app icon for the `.desktop` file and About, generated the same way as
    `macos/icon-from-screenshot.sh` does it (`still-webp` plus ImageMagick
    for the rounded corners), but writing PNGs instead of using the
    macOS-only `sips` and `iconutil`
  - Let the kernel batch our timer wakeups to save battery, like the macOS
    timer tolerance does (for example using `prctl(PR_SET_TIMERSLACK)`), and
    measure wakeups / power (for example using `powertop`), like the
    `powermetrics` section in `macos/README.md`
  - Build the Linux crate in GitHub CI, and run clippy there for both crates
  - Packaging and releases (macOS has `release.sh` and a Homebrew tap, and
    Homebrew also runs on Linux; other options are .deb, Flatpak, AUR, ...)
  - Mention Linux in the top level `README.md`
  - Anything found during Steps 3 and 4 (icon size, frame rate, CPU usage)
  - gnome-shell CPU usage. With LoadViz running at 10 Hz, gnome-shell uses
    about 12.7% CPU, against 2.85% without it. The cost is per update, not
    per pixel: it is the same for 16x16, 32x32 and 64x64 icons. The profile
    is spread over the AppIndicator extension's JavaScript, GLib main loop
    sources and GObject allocations, with no pixel copying hotspots. Options:
    a lower frame rate, or finding out what the extension does per update and
    whether a different way of updating the icon is cheaper.
  - `loadviz`'s own CPU usage (1.6% at 32x32) is mostly `ksni`/`zvariant`
    serializing the pixel data for D-Bus one byte at a time. Rendering is
    about 0.3% of the samples, and the RGB to ARGB conversion doesn't show up.
  - Show memory stalls using PSI (`/proc/pressure/memory`), which measures
    how much time tasks spend waiting for memory, swap-in included. This
    would bring back the swap signal lost by counting iowait as idle.
  - A 2x1 icon like on macOS. GNOME's AppIndicator extension draws pixmaps
    in a square, but shows a wide icon if `IconName` is the path of an image
    file (at least 1.5x as wide as high) in a user writable directory; that
    special case exists for indicator-multiload. It would mean writing a PNG
    10 times per second (to `$XDG_RUNTIME_DIR`, which is in RAM), probably
    alternating file names to get past gnome-shell's texture cache, and
    gnome-shell decoding a file per frame. It is GNOME specific; other tray
    hosts (KDE) are believed to draw icons square anyway.

## Step 7: Build and run on macOS (Johan)

Done by Johan on macOS, on this branch, before merging to `main`:

- Build and run the macOS app, and fix anything the `libloadviz` changes
  broke.
- Run `cargo test` in `libloadviz`, so that the shared CPU load test from
  Step 1 runs on macOS too.
- Delete this plan file.
