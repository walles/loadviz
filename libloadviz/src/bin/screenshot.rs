#![allow(clippy::needless_return)]

/// Writes a screenshot to a file
use std::{fs, mem, slice};

use libloadviz::renderer::Renderer;
use libwebp_sys::*;

fn main() {
    let width = 100;
    let height = 75;
    let frames_per_second = 10;
    let seconds = 10;
    let quality = 100.0; // 0-100, pick a number...

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

    let anim_params = WebPMuxAnimParams {
        bgcolor: 0,
        loop_count: 0, // 0 == infinite
    };
    // Ref: https://github.com/webmproject/libwebp/blob/08d60d60066eb30ab8e0e3ccfa0cd0b68f8cccc6/src/webp/mux.h#L423-L442
    let enc_options = WebPAnimEncoderOptions {
        anim_params,
        minimize_size: false as i32,
        kmin: 0,
        kmax: 0,                   // Up this number to get keyframes
        allow_mixed: false as i32, // "true" here gets us lossy frames all the time and they are ugly
        verbose: true as i32,
        padding: [0, 0, 0, 0],
    };
    let encoder = unsafe { WebPAnimEncoderNew(width as i32, height as i32, &enc_options) };

    // Ref: https://github.com/webmproject/libwebp/blob/08d60d60066eb30ab8e0e3ccfa0cd0b68f8cccc6/src/webp/encode.h#L94-L153
    let mut config = unsafe { mem::zeroed::<WebPConfig>() };
    assert!(0 != unsafe { WebPConfigPreset(&mut config, WEBP_PRESET_DEFAULT, quality) });
    config.lossless = true as i32; // All my attempts at lossy have been really bad

    for i in 0..(frames_per_second * seconds) {
        let dt_seconds = i as f64 / frames_per_second as f64;

        renderer.render_image(&loads, width, height, dt_seconds, &mut pixels);

        let dt_milliseconds = (dt_seconds * 1000.0) as i32;

        unsafe {
            let mut frame = mem::zeroed();
            assert!(0 != WebPPictureInit(&mut frame));
            frame.width = width as i32;
            frame.height = height as i32;
            assert!(0 != WebPPictureImportRGB(&mut frame, pixels.as_ptr(), 3 * width as i32));
            assert!(0 != WebPAnimEncoderAdd(encoder, &mut frame, dt_milliseconds, &config));
            WebPPictureFree(&mut frame);
        }
    }

    unsafe {
        // Encode!
        let mut data;
        data = mem::zeroed();
        WebPDataInit(&mut data);
        assert!(0 != WebPAnimEncoderAssemble(encoder, &mut data));

        // Output the result
        fs::write(filename, slice::from_raw_parts(data.bytes, data.size))
            .expect("Unable to write file");
    }
}
