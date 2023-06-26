use std::fs;

use libloadviz::renderer::Renderer;
use libwebp_sys::WebPEncodeRGB;

/// Writes a screenshot to a file

fn main() {
    // FIXME: Put this file in the same directory as Cargo.toml
    let filename = "screenshot.webp";

    let loads = vec![
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.3,
            system_0_to_1: 0.25,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.2,
            system_0_to_1: 0.20,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.1,
            system_0_to_1: 0.1,
        },
    ];

    let width = 100;
    let height = 75;
    let mut pixels = vec![0u8; width * height * 3];
    let renderer: Renderer = Default::default();
    renderer.render_image(&loads, width, height, &mut pixels);

    // Ref: https://crates.io/crates/libwebp-sys
    let encoded: Vec<u8>;
    unsafe {
        let mut out_buf = std::ptr::null_mut();
        let stride = width as i32 * 3;
        let encoded_length = WebPEncodeRGB(
            pixels.as_ptr(),
            width as i32,
            height as i32,
            stride,
            50.0,
            &mut out_buf,
        );
        encoded = std::slice::from_raw_parts(out_buf, encoded_length).into();
    }

    fs::write(filename, encoded).expect("Unable to write file");
}
