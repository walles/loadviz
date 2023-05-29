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
  
  // FIXME: We want RGB only, not RGBA, replace rawValue with something else
  let bitmapInfo:CGBitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedFirst.rawValue)
  
  let bitsPerComponent = 8 //number of bits in UInt8
  let bitsPerPixel = 3 * bitsPerComponent //RGB uses 3 components
  let bytesPerRow = bitsPerPixel * width / 8
  let providerRef = CGDataProvider(
    data:         NSData(bytes: pixels, length: height * bytesPerRow)
  )
  
  let cgim = CGImageCreate(
    width,
    height,
    bitsPerComponent,
    bitsPerPixel,
    bytesPerRow,
    rgbColorSpace,
    bitmapInfo,
    providerRef!,
    nil,
    true,
    .RenderingIntentDefault
  )
  
  return NSImage(CGImage: cgim!, size: NSSize(width: width, height: height))
}

struct ContentView: View {
  var body: some View {
    let width: UInt = 100
    let height: UInt = 100
    let imageBytes = LibLoadViz.get_image(width, height)!
    
    Image(nsImage: imageFromPixels(pixels: imageBytes, width: Int(width), height: Int(height)))
  }
}

struct ContentView_Previews: PreviewProvider {
  static var previews: some View {
    ContentView()
  }
}
