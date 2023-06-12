//
//  LoadVizApp.swift
//  LoadViz
//
//  Created by Johan Walles on 2023-06-10.
//

import SwiftUI

@main
struct LoadVizApp: App {
  var statusItem: NSStatusItem

  init() {
    // SwiftUI content view & a hosting view
    // Don't forget to set the frame, otherwise it won't be shown.
    let contentViewSwiftUI = VStack {
      Color.blue
      Text("Test Text")
      Color.white
    }
    let contentView = NSHostingView(rootView: contentViewSwiftUI)
    contentView.frame = NSRect(x: 0, y: 0, width: 200, height: 200)

    // Status bar icon SwiftUI view & a hosting view.
    let iconView = NSHostingView(rootView:
      LoadVizView(image: NSImage(size: NSSize(width: 40, height: 22))))
    iconView.frame = NSRect(x: 0, y: 0, width: 40, height: 22)

    // Creating a menu item & the menu to add them later into the status bar
    let menuItem = NSMenuItem()
    menuItem.view = contentView
    let menu = NSMenu()
    menu.addItem(menuItem)

    // Adding content view to the status bar
    let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    statusItem.menu = menu

    // Adding the status bar icon
    statusItem.button?.addSubview(iconView)
    statusItem.button?.frame = iconView.frame

    // StatusItem is stored as a property.
    self.statusItem = statusItem
  }

  var body: some Scene {
    WindowGroup {
      ContentView()
    }
  }
}
