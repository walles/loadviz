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
    //
    let contentViewSwiftUI = VStack {
      Color.blue
      Text("Test Text")
      Color.white
    }
    let contentView = NSHostingView(rootView: contentViewSwiftUI)
    contentView.frame = NSRect(x: 0, y: 0, width: 200, height: 200)

    // Status bar icon SwiftUI view & a hosting view.
    //
    let iconSwiftUI = ZStack(alignment: .center) {
      Rectangle()
        .fill(Color.green)
        .cornerRadius(5)
        .padding(2)

      Text("3")
        .background(
          Circle()
            .fill(Color.blue)
            .frame(width: 15, height: 15)
        )
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
        .padding(.trailing, 5)
    }
    let iconView = NSHostingView(rootView: iconSwiftUI)
    iconView.frame = NSRect(x: 0, y: 0, width: 40, height: 22)

    // Creating a menu item & the menu to add them later into the status bar
    //
    let menuItem = NSMenuItem()
    menuItem.view = contentView
    let menu = NSMenu()
    menu.addItem(menuItem)

    // Adding content view to the status bar
    //
    let statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    statusItem.menu = menu

    // Adding the status bar icon
    //
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
