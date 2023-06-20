import Cocoa
import ServiceManagement

// From: https://stackoverflow.com/a/38596649/473672
func imageFromPixels(pixels: UnsafePointer<UInt8>, width: Int, height: Int) -> NSImage {
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
    shouldInterpolate: true,
    intent: .defaultIntent
  )

  return NSImage(cgImage: cgim!, size: NSSize(width: width, height: height))
}

class AppDelegate: NSObject, NSApplicationDelegate {
  private var statusItem: NSStatusItem!
  private var libLoadViz = LibLoadViz.new_loadviz()

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    if let button = statusItem.button {
      Timer.scheduledTimer(withTimeInterval: 0.1, repeats: true) { [self] _ in
        button.image = getNewButtonImage()
      }
      button.image = getNewButtonImage()
    }

    setupMenus()
  }

  private func getNewButtonImage() -> NSImage {
    // FIXME: These are just some random numbers I got from
    // the Internet, what dimensions should we really use?
    let width = 40
    let height = 22

    return imageFromPixels(
      pixels: LibLoadViz.get_image(libLoadViz, UInt(width), UInt(height)),
      width: width,
      height: height
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
    // FIXME: let icon = bundle.image(forResource: "icon.png.icns")
    let aboutOptions: [NSApplication.AboutPanelOptionKey: Any] = [
      NSApplication.AboutPanelOptionKey.applicationName: "LoadViz",
      NSApplication.AboutPanelOptionKey.applicationVersion: version!,
      NSApplication.AboutPanelOptionKey.version: gitHash!,
      // FIXME: NSApplication.AboutPanelOptionKey.applicationIcon: icon!,
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
