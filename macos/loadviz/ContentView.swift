//
//  ContentView.swift
//  loadviz
//
//  Created by Johan Walles on 2023-05-29.
//

import SwiftUI

struct ContentView: View {
  var body: some View {
    let a = Int32(5)
    let b = Int32(3)
    let number = LibLoadViz.add(a, b)

    Text("\(a) + \(b) = \(number)")
      .padding()
  }
}

struct ContentView_Previews: PreviewProvider {
  static var previews: some View {
    ContentView()
  }
}
