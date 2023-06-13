// Ref: https://sarunw.com/posts/how-to-create-macos-app-without-storyboard/

import AppKit
import Cocoa
import Foundation

let app = NSApplication.shared
let delegate = AppDelegate()
app.delegate = delegate

_ = NSApplicationMain(CommandLine.argc, CommandLine.unsafeArgv)
