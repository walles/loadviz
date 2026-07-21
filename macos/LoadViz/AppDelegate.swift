import Cocoa
import LibLoadViz
import ServiceManagement

// From: https://stackoverflow.com/a/38596649/473672
//
// `width` and `height` are the bitmap's pixel dimensions, while `pointSize` is
// the size the resulting image should occupy on screen (in points). Rendering
// the bitmap at the display's native pixel density and then labelling it with
// the smaller point size lets AppKit draw it 1:1, so CoreGraphics never has to
// upscale the bitmap every frame.
func imageFromPixels(pixels: UnsafePointer<UInt8>, width: Int, height: Int, pointSize: NSSize) -> NSImage {
  let rgbColorSpace = CGColorSpaceCreateDeviceRGB()

  let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.none.rawValue)

  let bitsPerComponent = 8 // number of bits in UInt8
  let bitsPerPixel = 3 * bitsPerComponent // RGB uses 3 components
  let bytesPerRow = bitsPerPixel * width / 8
  let providerRef = CGDataProvider(
    data: NSData(bytes: pixels, length: height * bytesPerRow)
  )

  let cgim = CGImage(
    width: width,
    height: height,
    bitsPerComponent: bitsPerComponent,
    bitsPerPixel: bitsPerPixel,
    bytesPerRow: bytesPerRow,
    space: rgbColorSpace,
    bitmapInfo: bitmapInfo,
    provider: providerRef!,
    decode: nil,
    shouldInterpolate: false,
    intent: .defaultIntent
  )

  return NSImage(cgImage: cgim!, size: pointSize)
}

class AppDelegate: NSObject, NSApplicationDelegate {
  private var statusItem: NSStatusItem!
  private var libLoadViz = LibLoadViz.new_loadviz()

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    if let button = statusItem.button {
      let timer = Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [self] _ in
        if !(button.window?.occlusionState.contains(.visible))! {
          // Nobody can see our icon: the menu bar is auto-hidden, the screen is
          // locked, or another user is switched in via fast user switching.
          // Measurements confirmed occlusionState reports our button as not
          // visible in all three cases.
          // Ref: https://developer.apple.com/forums/thread/71171
          return
        }

        button.image = getNewButtonImage()
      }

      // Let macOS coalesce our 10 Hz wakeups with other timers to save battery.
      // Allowing slack up to the full interval means a fire may slip anywhere
      // within its 0.1s window, which can make the animation slightly less even.
      timer.tolerance = 0.1

      button.image = getNewButtonImage()
    }

    setupMenus()
  }

  private func getNewButtonImage() -> NSImage {
    // Height matches the menu bar exactly:
    // https://developer.apple.com/documentation/appkit/nsstatusbar/thickness
    let pointHeight = NSStatusBar.system.thickness

    let pointWidth = pointHeight * 2

    // Render at the display's native pixel density. On a 2x Retina screen this
    // makes the bitmap 88x44 px so AppKit can draw it 1:1 instead of upscaling
    // a 44x22 px bitmap every frame. Upscaling and rendering properly to begin
    // with both have roughly the same cost, but rendering correctly looks nicer.
    let scale = statusItem.button?.window?.backingScaleFactor
      ?? NSScreen.main?.backingScaleFactor
      ?? 2.0
    let pixelWidth = Int((pointWidth * scale).rounded())
    let pixelHeight = Int((pointHeight * scale).rounded())

    return imageFromPixels(
      pixels: LibLoadViz.get_image(libLoadViz, UInt(pixelWidth), UInt(pixelHeight)),
      width: pixelWidth,
      height: pixelHeight,
      pointSize: NSSize(width: pointWidth, height: pointHeight)
    )
  }

  private func setupMenus() {
    let menu = NSMenu()

    menu.addItem(
      NSMenuItem(
        title: "Help",
        action: #selector(help),
        keyEquivalent: ""
      ))

    menu.addItem(
      NSMenuItem(
        title: "About",
        action: #selector(about),
        keyEquivalent: ""
      ))

    menu.addItem(NSMenuItem.separator())

    menu.addItem(
      NSMenuItem(
        title: "Quit",
        action: #selector(quit),
        keyEquivalent: ""
      ))

    statusItem.menu = menu
  }

  @objc private func help() {
    NSWorkspace.shared.open(URL(string: "https://github.com/walles/loadviz/#readme")!)
  }

  @objc private func about() {
    let credits = NSAttributedString(
      string: "https://github.com/walles/loadviz",
      attributes: [
        NSAttributedString.Key.link: "https://github.com/walles/loadviz"
      ]
    )

    // The Git hash and version get filled in by a "Run Script" build step
    let bundle = Bundle(for: AppDelegate.self)
    let version = bundle.infoDictionary?["CFBundleShortVersionString"] as? String
    let gitHash = bundle.infoDictionary?["GitHash"] as? String
    let icon = bundle.image(forResource: "loadviz.icns")
    let aboutOptions: [NSApplication.AboutPanelOptionKey: Any] = [
      NSApplication.AboutPanelOptionKey.applicationName: "LoadViz",
      NSApplication.AboutPanelOptionKey.applicationVersion: version!,
      NSApplication.AboutPanelOptionKey.version: gitHash!,
      NSApplication.AboutPanelOptionKey.applicationIcon: icon!,
      NSApplication.AboutPanelOptionKey.credits: credits,
    ]
    NSApplication.shared.activate(ignoringOtherApps: true)
    NSApplication.shared.orderFrontStandardAboutPanel(options: aboutOptions)
  }

  @objc private func quit() {
    let isEnabled = SMLoginItemSetEnabled("com.gmail.walles.johan.LoadVizAutoLauncher" as CFString, false)
    if isEnabled {
      NSLog("Disabling LoadViz failed")
    }

    // FIXME: Why do we need two parentheses here?
    NSApplication.shared.terminate(_:)(_:0)
  }
}
