// Ref: https://sarunw.com/posts/how-to-create-macos-app-without-storyboard/

import AppKit
import Cocoa
import Foundation
import ServiceManagement

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate

let isEnabled = SMLoginItemSetEnabled("com.gmail.walles.johan.AutoLauncher" as CFString, true)
if !isEnabled {
  NSLog("Enabling LoadViz failed")
}

_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
