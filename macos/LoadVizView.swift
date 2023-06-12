import SwiftUI

// From: https://stackoverflow.com/a/38596649/473672
func imageFromPixels(pixels: UnsafePointer<UInt8>, width: Int, height: Int) -> NSImage {
  let rgbColorSpace = CGColorSpaceCreateDeviceRGB()

  // FIXME: Verify this gets us the correct colors
  let bitmapInfo = CGBitmapInfo(rawValue: CGImageAlphaInfo.none.rawValue)

  let bitsPerComponent = 8 // number of bits in UInt8
  let bitsPerPixel = 3 * bitsPerComponent // RGB uses 3 components
  let bytesPerRow = bitsPerPixel * width / 8
  let providerRef = CGDataProvider(
    data: NSData(bytes: pixels, length: height * bytesPerRow)
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

struct LoadVizView: View {
  /// Create using NSImage(size:) to get the right size of the output image
  @State var image: NSImage

  var loadviz = LibLoadViz.new_loadviz()

  // FIXME: If we se this to 0.1 then the demo app window never appears. 0.3 works.
  let timer = Timer.publish(every: 0.1, on: .main, in: .common).autoconnect()

  var body: some View {
    Image(
      nsImage: image
    ).onReceive(timer, perform: { _ in
      let width = UInt(image.size.width)
      let height = UInt(image.size.height)
      let imageBytes = LibLoadViz.get_image(loadviz, width, height)!
      image = imageFromPixels(pixels: imageBytes, width: Int(width), height: Int(height))
    })
  }
}

struct LoadVizView_Previews: PreviewProvider {
  static var previews: some View {
    LoadVizView(image: NSImage(size: NSSize(width: 100, height: 100)))
  }
}
