# LoadViz for Linux

A tray icon ([StatusNotifierItem]) showing LoadViz, written in Rust on top of
[`libloadviz`](../libloadviz/).

[StatusNotifierItem]: https://www.freedesktop.org/wiki/Specifications/StatusNotifierItem/

## Building and running

You need Rust, preferably installed using [rustup](https://rustup.rs/).

Build and run from this directory:

```sh
cargo run --release
```

`cargo run` rebuilds first if anything changed, including `libloadviz`. You
can also run the binary directly once built: `target/release/loadviz`.

Use `--release`: in debug builds, `libloadviz` replaces the real system load
with a hard coded demo cycle.

Quit using the icon's menu, which opens on both left and right click.

### GNOME

GNOME needs the [AppIndicator extension] to show tray icons. Ubuntu ships it
enabled. KDE Plasma shows tray icons out of the box.

[AppIndicator extension]: https://extensions.gnome.org/extension/615/appindicator-support/

### Icon size

The icon is 32x32 pixels: GNOME draws tray icons 16 logical pixels high, which
is 32 physical pixels on a 2x display. On a 2x display, GNOME draws it 1:1. On
other display scales it gets scaled.

The icon is square because tray hosts draw icons in a square. See the TODO list
for a 2x1 icon like on macOS.

## iowait counts as idle

On Linux, `libloadviz` counts iowait as idle time, not as system time.

Counting it as system time would have been nice, since waiting for swap would
show up there. But io_uring marks tasks waiting for completions as iowait, even
when no disk is involved. [Ghostty]'s event loop (libxev on io_uring) does that
all the time: with Ghostty running, two cores showed 100% system load on an
idle machine, and quitting Ghostty made that go away.

The cost: time spent waiting for swap-in reads from disk doesn't show. The CPU
side of swapping (page faults, `kswapd` and direct reclaim, zram / zswap
compression) is still system time.

This is the same as on macOS, which has no iowait counter.

The kernel lets programs opt out of this: `io_uring_enter()` takes an
`IORING_ENTER_NO_IOWAIT` flag, and `IORING_FEAT_NO_IOWAIT` tells whether the
kernel supports it. So the fix belongs in Ghostty's event loop library
([libxev](https://github.com/mitchellh/libxev)) rather than in the kernel.

[Ghostty]: https://ghostty.org/

## Profiling

LoadViz redraws its icon 10 times per second and is otherwise idle. Each frame
is rendered, converted from RGB to ARGB, and sent over D-Bus to the tray host,
which in GNOME's case is `gnome-shell`. So profile both processes.

Set `LOADVIZ_FORCE_MAX` in the environment to report every core as fully
loaded. This drives the renderer into its worst case and keeps the animation
deterministic, which makes A/B comparisons reliable:

```sh
LOADVIZ_FORCE_MAX=1 cargo run --release
```

### `loadviz` and `gnome-shell` together

No `sudo` needed, since both are your own processes:

```sh
perf record -g -p $(pgrep -x loadviz),$(pgrep -x gnome-shell) -- sleep 15
DEBUGINFOD_URLS=https://debuginfod.ubuntu.com perf report --sort comm,dso
```

`DEBUGINFOD_URLS` lets `perf report` download debug symbols for the GNOME
libraries. Ubuntu sets it in `/etc/profile.d`, but fish doesn't read that, so
set it explicitly. `gnome-shell`'s JavaScript frames still show up as
unknown addresses.

To see CPU usage rather than where it goes, compare `top` with and without
LoadViz running.

What the profile showed at 32x32, with a 2x display on Ubuntu:

- `loadviz` uses about 1.6% CPU. Most of that is `ksni` / `zvariant`
  serializing the pixel data for D-Bus, one byte at a time. Rendering is about
  0.3% of the samples, and the RGB to ARGB conversion doesn't show up.
- `gnome-shell` uses about 12.7% CPU with LoadViz running, against 2.85%
  without it. That cost is per update, not per pixel: it is the same for
  16x16, 32x32 and 64x64 icons. The profile is spread over the AppIndicator
  extension's JavaScript, GLib main loop sources and GObject allocations, with
  no pixel copying hotspots.

### Rust renderer, in isolation

See [the macOS README](../macos/README.md#rust-renderer-in-isolation) for the
background. On Linux, `perf` can start the benchmark binary itself:

```sh
cd ../libloadviz
cargo bench --no-run          # prints the bench binary's path
perf record -g -- target/release/deps/rendering-<hash> --bench --profile-time 30 "render 100x100 image"
perf report --no-children
```

## TODO

- `gnome-shell` CPU usage, see [Profiling](#profiling). Options: a lower frame
  rate, or making the AppIndicator extension
  ([ubuntu/gnome-shell-extension-appindicator](https://github.com/ubuntu/gnome-shell-extension-appindicator))
  cheaper per update. What is known so far:
  - Painting is the smaller part. gnome-shell's own timing marks
    (`sysprof-cli --gnome-shell`) put the compositor at about 1.7 ms per frame,
    and `clutter_frame_clock_dispatch` covers only about 20% of gnome-shell's
    `perf` samples. The rest is most likely the extension's JavaScript.
  - Per frame, the extension receives our `NewIcon` signal, waits 30 ms to
    batch signals, then fetches `IconName`, `IconPixmap` and
    `IconAccessibleDesc` using three separate D-Bus `Get` calls.
    `IconAccessibleDesc` fails every time since `ksni` doesn't implement it.
    Then it builds a new `St.ImageContent` through a chain of promises,
    cancellables and icon cache lookups.
  - Next step: a JavaScript profile, to see which of these costs. `perf` can't
    see JavaScript frames, and `sysprof-cli --gnome-shell` only records timing
    marks. A nested gnome-shell (running in a window) with a copy of the
    extension would allow both profiling and trying patches without logging
    out.
  - Moving the work out of JavaScript isn't realistic: extensions can't ship
    compiled code, and GNOME doesn't want tray icons in gnome-shell itself.
    The pixel work is already done in C (`St`, Clutter, Cogl); the JavaScript
    cost is the glue around it (promises, async D-Bus calls, and the
    JavaScript to C bridge, for example about 4% of gnome-shell's samples are
    under `gi_callable_info_create_closure`). So the fix would be to make the
    JavaScript do less per frame.
- `loadviz`'s own D-Bus serialization cost, see [Profiling](#profiling).
- Pick the icon size from the display scale, rather than always rendering
  32x32
- A 2x1 icon like on macOS. GNOME's AppIndicator extension draws pixmaps in a
  square, but shows a wide icon if `IconName` is the path of an image file (at
  least 1.5x as wide as high) in a user writable directory; that special case
  exists for indicator-multiload. It would mean writing a PNG 10 times per
  second (to `$XDG_RUNTIME_DIR`, which is in RAM), probably alternating file
  names to get past gnome-shell's texture cache, and gnome-shell decoding a
  file per frame. It is GNOME specific; other tray hosts (KDE) are believed to
  draw icons square anyway.
- Show memory stalls using PSI (`/proc/pressure/memory`), which measures how
  much time tasks spend waiting for memory, swap-in included. This would bring
  back the swap signal lost by [counting iowait as idle](#iowait-counts-as-idle).
- Report the iowait problem to libxev, see
  [iowait counts as idle](#iowait-counts-as-idle).
- Pause rendering when the icon isn't visible (screen locked etc.)
- Install script (macOS has `install.sh`)
- Autostart through `~/.config/autostart/*.desktop`. Like on macOS, starting
  enables autostart and Quit disables it. If we package as a Flatpak, use the
  Background portal instead, since a sandboxed app can't write that file.
- Help and About menu items. On macOS, About shows the version and Git hash.
- An app icon for the `.desktop` file and About, generated the same way as
  `macos/icon-from-screenshot.sh` does it (`still-webp` plus ImageMagick for
  the rounded corners), but writing PNGs instead of using the macOS-only `sips`
  and `iconutil`
- Let the kernel batch our timer wakeups to save battery, like the macOS timer
  tolerance does (for example using `prctl(PR_SET_TIMERSLACK)`), and measure
  wakeups / power (for example using `powertop`), like the `powermetrics`
  section in `macos/README.md`
- Build the Linux crate in GitHub CI, and run clippy there for both crates
- Packaging and releases (macOS has `release.sh` and a Homebrew tap, and
  Homebrew also runs on Linux; other options are .deb, Flatpak, AUR, ...)
- Mention Linux in the top level `README.md`
