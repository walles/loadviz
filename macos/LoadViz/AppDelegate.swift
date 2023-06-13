import Cocoa

class AppDelegate: NSObject, NSApplicationDelegate {
  private var statusItem: NSStatusItem!

  func applicationDidFinishLaunching(_ aNotification: Notification) {
    statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
    if let button = statusItem.button {
      button.image = NSImage(systemSymbolName: "1.circle", accessibilityDescription: "1")
    }

    setupMenus()
  }

  private func setupMenus() {
    let menu = NSMenu()

    let one = NSMenuItem(title: "One", action: #selector(didTapOne), keyEquivalent: "1")
    menu.addItem(one)

    let two = NSMenuItem(title: "Two", action: #selector(didTapTwo), keyEquivalent: "2")
    menu.addItem(two)

    let three = NSMenuItem(title: "Three", action: #selector(didTapThree), keyEquivalent: "3")
    menu.addItem(three)

    menu.addItem(NSMenuItem.separator())

    menu.addItem(NSMenuItem(title: "Quit", action: #selector(NSApplication.terminate(_:)), keyEquivalent: "q"))

    statusItem.menu = menu
  }

  private func changeStatusBarButton(number: Int) {
    if let button = statusItem.button {
      button.image = NSImage(systemSymbolName: "\(number).circle", accessibilityDescription: number.description)
    }
  }

  @objc func didTapOne() {
    changeStatusBarButton(number: 1)
  }

  @objc func didTapTwo() {
    changeStatusBarButton(number: 2)
  }

  @objc func didTapThree() {
    changeStatusBarButton(number: 3)
  }
}
