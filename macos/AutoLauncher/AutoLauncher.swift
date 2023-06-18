// This is the AutoLauncherAppDelegate.swift file from here:
// https://jogendra.dev/implementing-launch-at-login-feature-in-macos-apps

import Cocoa
import os.log

class AutoLauncher: NSObject, NSApplicationDelegate {

    struct Constants {
        static let mainAppBundleID = "com.gmail.walles.johan.LoadViz"
    }

    func applicationDidFinishLaunching(_ aNotification: Notification) {
        let runningApps = NSWorkspace.shared.runningApplications
        let isRunning = runningApps.contains {
            $0.bundleIdentifier == Constants.mainAppBundleID
        }

        if !isRunning {
            var path = Bundle.main.bundlePath as NSString

            // This LoadVizAutoLauncher app target is actually embedded inside the
            // main application bundle under the subdirectory Contents/Library/LoginItems
            // (we are going to do this later). So including the helper app target name
            // (LoadVizAutoLauncher.app) there will be a total of 4 path components to be
            // deleted. That’s why we are looping 4 times.
            for _ in 1...4 {
                path = path.deletingLastPathComponent as NSString
            }

            let applicationPathString = path as String
            let pathURL = URL(fileURLWithPath: applicationPathString)
            NSWorkspace.shared.openApplication(at: pathURL,
                                               configuration: NSWorkspace.OpenConfiguration(),
                                               completionHandler: {_,err in
              if err != nil {
                os_log(.error, "Failed to launch LoadViz at %{public}@: %{public}@", pathURL.description, err!.localizedDescription)
              }
            })
        }
    }
}
