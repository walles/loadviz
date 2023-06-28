#![allow(clippy::needless_return)]

/// Writes a screenshot to a file
use std::fs;

use libloadviz::renderer::Renderer;
use webp_animation::prelude::*;

fn main() {
    let width = 100;
    let height = 75;
    let frames_per_second = 10;
    let seconds = 10;

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

    let mut pixels = vec![0u8; width * height * 3];
    let renderer: Renderer = Default::default();

    // FIXME: Add something about looping the animation
    let mut encoder = Encoder::new_with_options(
        (width as u32, height as u32),
        EncoderOptions {
            minimize_size: false, // FIXME: True?
            kmin: 0,
            kmax: 0, // FIXME: Up this to get keyframes
            allow_mixed: true,
            verbose: true,
            color_mode: ColorMode::Rgba,
            encoding_config: Some(EncodingConfig {
                encoding_type: EncodingType::Lossy(Default::default()),
                quality: 80.0,
                method: 6,
            }),
        },
    )
    .unwrap();

    for i in 0..(frames_per_second * seconds - 1) {
        let dt_seconds = i as f64 / frames_per_second as f64;

        renderer.render_image(&loads, width, height, dt_seconds, &mut pixels);

        let dt_milliseconds = (dt_seconds * 1000.0) as i32;
        encoder
            .add_frame(&rgb_to_rgba(&pixels), dt_milliseconds)
            .unwrap();
    }

    let final_milliseconds = frames_per_second * seconds * 1000;
    let webp_data = encoder.finalize(final_milliseconds).unwrap();
    fs::write(filename, webp_data).expect("Unable to write file");
}

// Workaround for: https://github.com/blaind/webp-animation/issues/11
fn rgb_to_rgba(pixels: &[u8]) -> Vec<u8> {
    let mut rgba = Vec::with_capacity(pixels.len() * 4 / 3);
    for pixel in pixels.chunks_exact(3) {
        rgba.extend_from_slice(pixel);
        rgba.push(255);
    }
    return rgba;
}
