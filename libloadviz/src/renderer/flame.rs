use std::ops::Range;

use crate::cpuload::CpuLoad;

use super::{get_load, interpolate, pixel_to_fraction, Renderer, BG_COLOR_RGB};

// Blackbody RGB values from: http://www.vendian.org/mncharity/dir3/blackbody/
static USER_LOAD_COLOR_RGB_WARMER: &[u8; 3] = &[0xff, 0xb4, 0x6b]; // 3000K
static USER_LOAD_COLOR_RGB_COOLER: &[u8; 3] = &[0xff, 0x38, 0x00]; // 1000K

/// Higher numbers => more detail
static SCALE: f32 = 2.0;

/// What fraction of the inside of the fire fades towards transparent?
static TRANSPARENT_INTERNAL_0_TO_1: f32 = 0.3;

/// How warped is the internal base image? Try setting `DISTORTION_DETAIL` to
/// zero to see the effect of changing this number.
static INTERNAL_DETAIL: f32 = 2.0 * SCALE;

/// How uneven are the fire's edges? Also, how much warping happens to the
/// internal base image?
///
/// Note that even if this is set to zero, the internal base image will still
/// move around, since time is one basis for the noise.
static DISTORTION_DETAIL: f32 = 3.0 * SCALE;

/// How many pixels wide is the distortion effect? This is in percent of the
/// shortest image dimension.
static DISTORTION_PIXEL_PERCENT: f32 = 10.0 / SCALE;

/// Higher values make the fire's edges softer
static ANTIALIAS_HEIGHT_0_TO_1: f32 = 0.1;

/// Without scaling the flames, 50% actual load looks more like 40% to the user
static HEIGHT_SCALE: f32 = 1.3;

impl Renderer {
    pub(super) fn get_flame_pixel(
        &self,
        viz_loads: &Vec<CpuLoad>,
        dt_seconds: f32,
        pixel_x: usize,
        pixel_y_from_bottom: usize,
        width: usize,
        height: usize,
    ) -> Option<[u8; 3]> {
        let internal_detail = INTERNAL_DETAIL / width as f32;
        let distortion_detail = DISTORTION_DETAIL / width as f32;
        let distortion_pixel_radius = width.min(height) as f32 * DISTORTION_PIXEL_PERCENT / 100.0;

        // Check whether we should even try to do flames maths. This improves
        // our idle-system benchmark by 63%.
        let highest_load_0_to_1 = HEIGHT_SCALE
            * viz_loads
                .iter()
                .map(|load| load.user_0_to_1)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();
        let highest_possible_flame_height_pixels =
            highest_load_0_to_1 * height as f32 + distortion_pixel_radius;
        if pixel_y_from_bottom as f32 > highest_possible_flame_height_pixels {
            // We're above all flames, no need for any (costly) noise maths
            //
            // This check improves our idle-system benchmark by 63%.
            return None;
        }

        // Noise output is -1 to 1, deciphered from here:
        // https://github.com/amethyst/bracket-lib/blob/0d2d5e6a9a8e7c7ae3710cfef85be4cab0109a27/bracket-noise/examples/simplex_fractal.rs#L34-L39
        let noise_x_m1_to_1 = self.noise.get_noise(
            distortion_detail * pixel_x as f32,
            //
            // Note that the time animation constant is by design different from
            // the one we use for temperature. This is supposed to help make the
            // flames look more organic.
            distortion_detail * pixel_y_from_bottom as f32 - dt_seconds * 2.5,
        );

        // Pick the load to show
        let dx_pixels = noise_x_m1_to_1 * distortion_pixel_radius;
        let distorted_pixel_x = pixel_x as f32 + dx_pixels;
        let x_fraction_0_to_1 = pixel_to_fraction(distorted_pixel_x, width);
        let cpu_load = get_load(viz_loads, x_fraction_0_to_1);
        let load_0_to_1 = cpu_load.user_0_to_1 * HEIGHT_SCALE;

        let highest_possible_flame_height_pixels =
            load_0_to_1 * height as f32 + distortion_pixel_radius;
        if pixel_y_from_bottom as f32 > highest_possible_flame_height_pixels {
            // We're above the flames at this particular column, no need for any
            // more (costly) noise maths.
            //
            // This check improves our busy benchmark by 5%.
            return None;
        }

        let noise_y_m1_to_1 = self.noise.get_noise(
            distortion_detail * pixel_x as f32,
            distortion_detail * pixel_y_from_bottom as f32,
        );

        // Figure out how to color the current pixel
        let dy_pixels = noise_y_m1_to_1 * distortion_pixel_radius;
        let distorted_pixel_y = pixel_y_from_bottom as f32 + dy_pixels;
        let y_from_bottom_0_to_1 = pixel_to_fraction(distorted_pixel_y, height);
        if y_from_bottom_0_to_1 > load_0_to_1 {
            return None;
        }

        // Get a 0-1 noise value for this coordinate, that scrolls up with time
        let temperature_0_to_1 = map_range(
            self.noise.get_noise(
                internal_detail * distorted_pixel_x,
                //
                // Note that the time animation constant is by design different
                // from the one we use for noise_x. This is supposed to help
                // make the flames look more organic.
                internal_detail * distorted_pixel_y - dt_seconds * 2.0,
            ),
            -1.0..1.0,
            0.0..1.0,
        );

        // Make the fire cooler the closer the top of the flame we get
        let temperature_0_to_1 =
            temperature_0_to_1 * get_cooling_factor(y_from_bottom_0_to_1, load_0_to_1);

        return Some(get_color_by_temperature(temperature_0_to_1));
    }
}

fn map_range(value: f32, from: Range<f32>, to: Range<f32>) -> f32 {
    return (value - from.start) * (to.end - to.start) / (from.end - from.start) + to.start;
}

/// Lower values mean lower temperatures
fn get_cooling_factor(y_from_bottom_0_to_1: f32, load_0_to_1: f32) -> f32 {
    let edge_cooling_factor = map_range(
        y_from_bottom_0_to_1,
        (load_0_to_1 - ANTIALIAS_HEIGHT_0_TO_1)..load_0_to_1,
        1.0..0.0,
    )
    .clamp(0.0, 1.0);

    let fraction_of_current_height = y_from_bottom_0_to_1 / load_0_to_1;
    let fraction_cooling_factor = 1.0 - fraction_of_current_height;

    return edge_cooling_factor.min(fraction_cooling_factor);
}

fn get_color_by_temperature(temperature_0_to_1: f32) -> [u8; 3] {
    if temperature_0_to_1 < TRANSPARENT_INTERNAL_0_TO_1 {
        return interpolate(
            temperature_0_to_1 / TRANSPARENT_INTERNAL_0_TO_1,
            BG_COLOR_RGB,
            USER_LOAD_COLOR_RGB_COOLER,
        );
    }

    return interpolate(
        (temperature_0_to_1 - TRANSPARENT_INTERNAL_0_TO_1) / (1.0 - TRANSPARENT_INTERNAL_0_TO_1),
        USER_LOAD_COLOR_RGB_COOLER,
        USER_LOAD_COLOR_RGB_WARMER,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flame_reaches_the_top() {
        let viz_loads = vec![CpuLoad {
            user_0_to_1: 1.0,
            system_0_to_1: 0.0,
        }];
        let renderer: Renderer = Default::default();
        let height = 100;
        let pixel = renderer.get_flame_pixel(&viz_loads, 0.0, 0, height - 1, 1, height);
        assert!(pixel.is_some());
    }
}
