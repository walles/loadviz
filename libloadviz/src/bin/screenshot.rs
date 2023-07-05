#![allow(clippy::needless_return)]

use std::{ffi::c_int, fs, mem, slice};

use libloadviz::renderer::Renderer;
use libwebp_sys::*;

struct AnimWriter {
    filename: String,

    pub width: usize,
    pub height: usize,

    encoder: *mut WebPAnimEncoder,
    config: WebPConfig,
}

impl AnimWriter {
    fn new(filename: &str, quality: f32, width: usize, height: usize) -> AnimWriter {
        let anim_params = WebPMuxAnimParams {
            bgcolor: 0,
            loop_count: 0, // 0 == infinite
        };

        // Ref: https://github.com/webmproject/libwebp/blob/08d60d60066eb30ab8e0e3ccfa0cd0b68f8cccc6/src/webp/mux.h#L423-L442
        let enc_options = WebPAnimEncoderOptions {
            anim_params,
            minimize_size: false as i32,
            kmin: 0,
            kmax: 0, // Up this number if you want keyframes
            allow_mixed: true as i32,
            verbose: false as i32,
            padding: [0, 0, 0, 0],
        };
        let encoder = unsafe { WebPAnimEncoderNew(width as i32, height as i32, &enc_options) };

        let mut config = unsafe { mem::zeroed::<WebPConfig>() };
        assert!(0 != unsafe { WebPConfigPreset(&mut config, WEBP_PRESET_DEFAULT, quality) });

        AnimWriter {
            filename: filename.to_owned(),
            width,
            height,
            encoder,
            config,
        }
    }

    fn add_frame(&mut self, dt_milliseconds: c_int, pixels: &[u8]) {
        unsafe {
            let mut frame = mem::zeroed();
            assert!(0 != WebPPictureInit(&mut frame));
            frame.width = self.width as i32;
            frame.height = self.height as i32;
            assert!(0 != WebPPictureImportRGB(&mut frame, pixels.as_ptr(), 3 * self.width as i32));
            assert!(
                0 != WebPAnimEncoderAdd(self.encoder, &mut frame, dt_milliseconds, &self.config)
            );
            WebPPictureFree(&mut frame);
        }
    }
}

impl Drop for AnimWriter {
    /// Save the animation to disk
    fn drop(&mut self) {
        unsafe {
            // Encode!
            let mut data;
            data = mem::zeroed();
            WebPDataInit(&mut data);
            assert!(0 != WebPAnimEncoderAssemble(self.encoder, &mut data));

            // Output the result
            fs::write(&self.filename, slice::from_raw_parts(data.bytes, data.size))
                .expect("Unable to write file");
        }
    }
}

/// Writes a screenshot to a file
///
/// See also `stillimage.rs`.
fn main() {
    let mut anim_writer = AnimWriter::new(
        // FIXME: Put this file in the same directory as Cargo.toml
        "screenshot.webp",
        90.0, // 0-100, pick a number...
        100,
        75,
    );
    let frames_per_second = 10;
    let seconds = 10;
    let xfade_seconds = 2;

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

    let mut pixels = vec![0u8; anim_writer.width * anim_writer.height * 3];
    let renderer: Renderer = Default::default();

    for i in 0..(frames_per_second * seconds) {
        let dt_seconds = i as f32 / frames_per_second as f32;

        if (dt_seconds as i32) < seconds - xfade_seconds {
            // No crossfade yet, just render one image
            renderer.render_image(
                &loads,
                anim_writer.width,
                anim_writer.height,
                dt_seconds,
                &mut pixels,
            );
        } else {
            // Render image 1
            let mut pixels1 = vec![0u8; anim_writer.width * anim_writer.height * 3];
            renderer.render_image(
                &loads,
                anim_writer.width,
                anim_writer.height,
                dt_seconds,
                &mut pixels1,
            );

            // Render image 2
            let mut pixels2 = vec![0u8; anim_writer.width * anim_writer.height * 3];
            renderer.render_image(
                &loads,
                anim_writer.width,
                anim_writer.height,
                // Render image before the first frame of the whole animation
                dt_seconds - (seconds as f32),
                &mut pixels2,
            );

            // Crossfade the two images into pixels
            for i in 0..(anim_writer.width * anim_writer.height * 3) {
                let p1 = pixels1[i];
                let p2 = pixels2[i];
                let seconds_into_crossfade = dt_seconds - (seconds as f32 - xfade_seconds as f32);
                let crossfade_0_to_1 = seconds_into_crossfade / xfade_seconds as f32;
                let weight1 = 1.0 - crossfade_0_to_1;
                let weight2 = crossfade_0_to_1;
                pixels[i] = (weight1 * p1 as f32 + weight2 * p2 as f32) as u8;
            }
        }

        let dt_milliseconds = (dt_seconds * 1000.0) as i32;
        anim_writer.add_frame(dt_milliseconds, &pixels);
    }
}
