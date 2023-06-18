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
        title: "Quit",
        action: #selector(quit),
        keyEquivalent: ""
      ))

    statusItem.menu = menu
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
