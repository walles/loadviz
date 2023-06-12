import SwiftUI

@main
struct demoApp: App {
  var body: some Scene {
    WindowGroup {
      LoadVizView(image: NSImage(size: NSSize(width: 100, height: 100)))
    }
  }
}
