//
//  ContentView.swift
//  loadviz
//
//  Created by Johan Walles on 2023-05-29.
//

import SwiftUI

// From: https://stackoverflow.com/a/38596649/473672
func imageFromPixels(pixels: UnsafePointer<UInt8>, width: Int, height: Int)-> NSImage {
  let rgbColorSpace = CGColorSpaceCreateDeviceRGB()

  // FIXME: Verify this gets us the correct colors
  let bitmapInfo:CGBitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.none.rawValue)

  let bitsPerComponent = 8 //number of bits in UInt8
  let bitsPerPixel = 3 * bitsPerComponent //RGB uses 3 components
  let bytesPerRow = bitsPerPixel * width / 8
  let providerRef = CGDataProvider(
    data:         NSData(bytes: pixels, length: height * bytesPerRow)
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

struct ContentView: View {
  var body: some View {
    let width: UInt = 100
    let height: UInt = 100

    // FIXME: We should do this once at startup and then reuse the same loadviz for all get_image() calls
    let fixme = LibLoadViz.new_loadviz()

    let imageBytes = LibLoadViz.get_image(fixme, width, height)!

    Image(nsImage: imageFromPixels(pixels: imageBytes, width: Int(width), height: Int(height)))
  }
}

struct ContentView_Previews: PreviewProvider {
  static var previews: some View {
    ContentView()
  }
}
