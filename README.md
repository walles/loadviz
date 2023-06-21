# Load Visualizer

Flames from the bottom show user load. Flames from the top show system load. The
most loaded cores burn in the middle.

So if the flames are high in the middle it means that one / a few cores are
maxed out. If the flames are sort of high everywhere it means that all cores are
working.

And if you have lots of system load, it can mean that you're short on RAM /
swapping.

The Load Visualizer provides a visualization of the current system load,
covering CPU usage and RAM pressure.

System and User CPU load are measured for each logical core of the system. RAM
pressure has to be inferred:

- If the user part is high on one CPU, getting CPUs with better single core
  performance might help.

- If the user part is high on lots of CPUs, getting more cores might help.

- If the system part of the CPU load is high, this can mean that your system is
  swapping, and that getting more RAM might help.

## Parts

There's one macOS app done in Swift and one library done in Rust.

## Rendering

Red flames from the bottom indicate user load. Green flames from the top
indicate system load.

The from-the-ceiling flames are in the way of the from-the-floor flames, just
like the system load is in the way of the user load.

## Why not measure RAM / swap usage?

People commonly think measuring RAM / swap usage will help them decide...

- When to close apps / tabs
- When to buy more RAM
- When to get faster swap

... but the truth is it won't help with any of that. Conclusions drawn from
those numbers will generally be wrong, and presenting them would just be
confusing.

### Swapping out unused memory

Let's say you have 8GB RAM, but you're using 25GB. 6GB is resident, 19GB is
swapped out and 2GB RAM is "free" / used as disk cache. Also, let's say that
only 4GB of RAM is actually needed right now, and the other 21GB just happen to
be around but are not going to be used during the near future.

Any RAM / Swap visualization would show this as critical because you're silly
high over quota.

But in reality, since you aren't using most of the memory, this is fine.

### Cost of swapping

Let's say you have two computers, both doing the same thing and having too little RAM.

One of them is swapping to a (slow) hard drive, and one to a (fast) SSD.

Then, even with the exact same memory numbers, the SSD machine will be in a lot
better situation than the HD machine.

How would you visualize this?

## Releasing a new version of the macOS status bar app

1. In Xcode, at the top of the screen, select "LoadViz"
1. In the menu bar, select "Product" -> "Archive"
1. In the new Archives screen, right click your new build and "Show in Finder"
1. Right click the `.xcarchive` file and select "Show Package Contents"
1. In the new Finder window, find your release build under "Products" ->
   "Applications"

## TODO

- Release the first public version on GitHub
- Package using HomeBrew
- Stop using CPU when the menu bar isn't visible
- Add a screenshot to the top of the README.md file, close to the explanation of
  how to read the display
- Make an icon
- Make sure the icon is visible in the Finder
- Make sure the icon is visible in the About dialog
- Make a Dock icon visualization
- Poll battery charge state as well
- Visualize battery charge as a blue sky (full) or a starry night (empty)
- Enable resizing the demo app. Resizing should scale the image so we can see it
  pixel by pixel. Nearest-neighbor scaling preferred.
- Find a _good_ way of getting the demo image to update at 10 FPS
- Figure out whether `cargo build` should be done in the `libloadviz` project
  instead. What we want is for the Rust build to run every time we press the
  Play button in Xcode, and for Xcode to understand whether either the bridging
  header or the static library have changed.

### Done

- Make the library export some C API that gets called by the Swift app
- Make the library return an image, any image
- Make a macOS app rendering this image in a window
- Make the demo app not just show black all the time even though the image from
  Rust is red
- Make demo binary update its image at 3 FPS
- Add `cargo build` of our library to the Xcode project's build process
- Make demo binary update its image at 10 FPS
- Draw some sort of bar chart
- Base bar chart on fake data for one CPU
- Make sure that data looks like CPU load (user / system / idle) and that the
  bars are based on that
- Make sure we can render everything from one CPU to more CPUs than the width of
  the display
- Make the bars look like sound visualizations
- Add test making sure `mirror_sort()` doesn't fail when two values are equal
- Replace the data with actual CPU load
- Test that we can render a zero-length CPU-loads array without crashing
- Make bars slide into place when metrics are updated
- Add micro benchmarks for the rendering code
- Profile the micro benchmarks and make the code faster
- Make a desktop app
- Make the desktop app show a menu bar icon
- Update the menu bar icon at 10Hz with the system load
- Don't hard code the dimensions in LoadVizView
- Make clicking the menu bar icon bring up a menu
- Add a "Quit" menu item
- Hide the desktop app icon from the Dock
- Hide the desktop app from cmd-tab
- Profile our release build to see if we can make it faster
- Make the desktop app start automatically on login:
  <https://jogendra.dev/implementing-launch-at-login-feature-in-macos-apps>
- Make sure our auto launcher doesn't pollute the global namespace
- Make sure the visualization doesn't start out as all-black
- Replace the whole visualization with red user flames from the bottom and green
  system flames from the top
- Test that `diff()` handles wrapping CPU counters correctly
- Get Git metadata into the release archive
- Add an "About" menu item doing something helpful. At least link to the GitHub
  project and show some version number.
- Run (a release build!) by default on Johan's desktop
- Add a Help menu item explaining how to read the visualization
- Make sure the visualization doesn't go to 100% after showing the About dialog
- Make sure we can install a packaged LoadViz on some other machine
- Add a license
