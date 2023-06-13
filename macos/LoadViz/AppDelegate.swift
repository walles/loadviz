//
//  AppDelegate.swift
//  LoadViz
//
//  Created by Johan Walles on 2023-06-13.
//

import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
  private var statusItem: NSStatusItem!

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    if let button = statusItem.button {
      button.image = NSImage(systemSymbolName: "1.circle", accessibilityDescription: "1")
    }
  }
}
