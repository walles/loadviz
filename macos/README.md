# LoadViz for macOS

To install from source, run `./install.sh`.

## Profiling

LoadViz redraws its menu bar icon on a timer and is otherwise idle: a sample of
the running app is over 99% blocked threads. So the first-order battery lever is
how *often* it draws — the frame count scales the rendering, the image handoff,
and the timer wakeups together.

Per frame, the CPU work is dominated by the Rust renderer. Sampling the app
under full load (`LOADVIZ_FORCE_MAX`, below) puts roughly four fifths of the
on-CPU time in rendering, about a tenth in handing the image to the status item
(`-[NSStatusBarButton setImage:]`), and almost none in building the CGImage.
Rendering is load-dependent, though: cheap when the system is idle (few flames
to draw), heavier when it is busy.

First make an ad-hoc signed release build:

```sh
xcodebuild -scheme LoadViz -configuration Release \
  CODE_SIGN_IDENTITY="-" CODE_SIGN_STYLE=Manual DEVELOPMENT_TEAM="" build
```

Set `LOADVIZ_FORCE_MAX` in the environment to report every core as fully loaded.
This drives the renderer into its worst case and keeps the animation
deterministic, which makes A/B comparisons reliable:

```sh
LOADVIZ_FORCE_MAX=1 path/to/LoadViz.app/Contents/MacOS/LoadViz
```

### Rust renderer, in isolation

Pixel rendering lives in the Rust crate, where you can profile it directly —
free of machine noise and the rest of the app. `sample` needs a running process
to attach to, so start the bench binary in a profiling loop and sample it:

```sh
cd ../libloadviz
cargo bench --no-run          # prints the bench binary's path
```

Grab the `rendering-<hash>` path it reports under `target/release/deps/`, then
run one benchmark in a long loop and sample it while it churns:

```sh
target/release/deps/rendering-<hash> --bench --profile-time 30 "render 100x100 image" &
sample $! 20 -file out.txt
```

`--profile-time` tells Criterion to skip its warmup and analysis and just run
the render loop, so the samples are almost all the pixel work. Read the leaf
histogram under "Sort by top of stack", same as for the app.

To measure whether a change actually paid off,
[Criterion](https://github.com/criterion-rs/criterion.rs)'s `cargo bench` times
the same renderer.

### On-CPU hotspots in the app

```sh
sample LoadViz 10 -file out.txt
```

Read the leaf histogram under "Sort by top of stack". Two things to keep in
mind:

- The process is idle between frames, so most samples are blocked threads.
  `sample` shows what the app does *while on CPU*, not how much CPU it uses.
- `sample` undercounts blocking calls. A synchronous IPC that waits for a reply
  spends its time blocked, not on CPU, so it barely appears here even when it
  costs a wakeup every frame. Time suspected IPC directly to catch those.

### Energy and wakeups

```sh
sudo powermetrics --samplers tasks,cpu_power,gpu_power \
  --show-process-energy -i 1000 -n 12 -o out.txt
```

This is the closest proxy for battery: look at the process's energy impact and
its wakeups per second, which track the timer. Compositing our icon is work the
`WindowServer` does on our behalf, charged to its process rather than ours — so
we could load it without it showing in our own numbers, but currently our tiny
icon is a negligible part of its load.
