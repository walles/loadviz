# LoadViz for macOS

To install from source, run `./install.sh`.

## Profiling

LoadViz redraws its menu bar image on a timer, so its cost is dominated by
per-frame rendering. To measure it, first make an ad-hoc signed release build:

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

See where CPU time goes (own process, no `sudo` needed); the leaf histogram is
under "Sort by top of stack":

```sh
sample LoadViz 10 -file out.txt
```

Measure per-process energy and wakeups (needs `sudo`). The `WindowServer` column
is system-wide, so keep the machine otherwise idle while sampling:

```sh
sudo powermetrics --samplers tasks,cpu_power,gpu_power \
  --show-process-energy -i 1000 -n 12 -o out.txt
```
