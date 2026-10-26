#![allow(clippy::needless_return)]

/// Renders a single high-res still frame to the file named on the command line.
/// Used as the source image for the macOS app icon; see
/// `macos/icon-from-screenshot.sh`.
use std::{env, fs, process::ExitCode, slice};

use libloadviz::renderer::Renderer;
use libwebp_sys::*;

fn main() -> ExitCode {
    let width = 1024;
    let height = 1024;
    let quality = 80.0; // 0-100, pick a number...

    if env::args().len() != 2 {
        println!("Usage: {} <output.webp>", env::args().next().unwrap());
        return ExitCode::FAILURE;
    }
    let filename = env::args().nth(1).unwrap();

    let loads = vec![
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.4,
            system_0_to_1: 0.4,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.2,
            system_0_to_1: 0.3,
        },
        libloadviz::cpuload::CpuLoad {
            user_0_to_1: 0.1,
            system_0_to_1: 0.1,
        },
    ];

    let mut pixels = vec![0u8; width * height * 3];
    let renderer = Renderer::with_width_based_details();
    renderer.render_image(&loads, width, height, 0.0, &mut pixels);

    let mut data: *mut u8 = std::ptr::null_mut();
    let size = unsafe {
        WebPEncodeRGB(
            pixels.as_ptr(),
            width as i32,
            height as i32,
            3 * width as i32,
            quality,
            &mut data,
        )
    };

    unsafe {
        fs::write(filename, slice::from_raw_parts(data, size)).expect("Unable to write file");
    }

    return ExitCode::SUCCESS;
}
