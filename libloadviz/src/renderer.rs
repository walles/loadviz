use std::time::Instant;

// FIXME: Maybe check if this library is faster?
// https://github.com/amethyst/bracket-lib/tree/master/bracket-noise
//
// Check both Simplex and Perlin.
use noise::{NoiseFn, Perlin};

use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];

// Blackbody RGB values from: http://www.vendian.org/mncharity/dir3/blackbody/
static USER_LOAD_COLOR_RGB_COOLER: &[u8; 3] = &[0xff, 0x38, 0x00]; // 1000K
static USER_LOAD_COLOR_RGB_WARMER: &[u8; 3] = &[0xff, 0xe4, 0xce]; // 5000K

static CLOUD_COLOR_DARK: &[u8; 3] = &[0x88, 0x88, 0x88];
static CLOUD_COLOR_BRIGHT: &[u8; 3] = &[0xff, 0xff, 0xff];

/// How much of the cloud should fade towards transparent? Higher is more
/// transparency, 0 means no transparency = a sharp edge to the cloud.
static CLOUD_TRANSPARENT_FRACTION: f32 = 0.5;

pub struct Renderer {
    perlin: Perlin,
    t0: Instant,
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            perlin: Perlin::new(Perlin::DEFAULT_SEED),
            t0: Instant::now(),
        }
    }
}

impl Renderer {
    /// Don't call this! It's public for benchmarking purposes only.
    ///
    /// You should call `LoadViz::render_image()` instead.
    pub fn render_image(
        &self,
        currently_displayed_loads: &Vec<CpuLoad>,
        width: usize,
        height: usize,
        pixels: &mut Vec<u8>,
    ) {
        let distortion_pixel_radius = 3.0;

        if currently_displayed_loads.is_empty() {
            // FIXME: Draw something nice?
            return;
        }
        let viz_loads = mirror_sort(currently_displayed_loads);

        for i in (0..pixels.len()).step_by(3) {
            let base_x = (i / 3) % width;
            let y_from_top = (i / 3) / width;

            // Higher scale number = more details.
            let scale = 20.0 / width as f64;
            let dt: f64 = self.t0.elapsed().as_secs_f64();

            // NOTE: Experiments show that the Perlin output is -1 to 1
            let dx_m1_to_1 =
                self.perlin
                    .get([scale * base_x as f64, scale * y_from_top as f64, dt]);
            let dx_pixels = dx_m1_to_1 * distortion_pixel_radius;
            let dy_m1_to_1 =
                self.perlin
                    .get([scale * base_x as f64, scale * y_from_top as f64, -dt - 1.0]);
            let dy_pixels = dy_m1_to_1 * distortion_pixel_radius;
            let distorted_x: usize = ((base_x as f64 + dx_pixels).clamp(0.0, width as f64 + 1.0)
                as usize)
                .min(width - 1);
            let distorted_y: f64 = (height - y_from_top - 1) as f64 + dy_pixels;

            let cpu_load = &viz_loads[(distorted_x * viz_loads.len()) / width];

            let y_height = distorted_y as f32 / height as f32;
            let color = if let Some(cloud_color) =
                get_cloud_pixel(&viz_loads, base_x, y_from_top, width, height, dx_m1_to_1)
            {
                cloud_color
            } else if y_height > cpu_load.user_0_to_1 {
                *BG_COLOR_RGB
            } else {
                // FIXME: The top 10% (?) of the flames should fade towards the
                // background color. This should make the flames look more
                // transparent and less artificial.
                let fraction = y_height / cpu_load.user_0_to_1;
                interpolate(
                    fraction as f64,
                    USER_LOAD_COLOR_RGB_WARMER,
                    USER_LOAD_COLOR_RGB_COOLER,
                )
            };

            pixels[i] = color[0];
            pixels[i + 1] = color[1];
            pixels[i + 2] = color[2];
        }
    }
}

fn get_cloud_pixel(
    viz_loads: &Vec<CpuLoad>,
    pixel_x: usize,
    pixel_y_from_top: usize,
    width: usize,
    height: usize,
    noise_m1_to_1: f64,
) -> Option<[u8; 3]> {
    // Use the x coordinate to decide which load to use
    let load = &viz_loads[(pixel_x * viz_loads.len()) / width];
    if load.system_0_to_1 < 0.01 {
        // Prevent a division by zero below
        return None;
    }

    // Compute the sysload height for this load
    let height_pixels = load.system_0_to_1 * height as f32;

    if pixel_y_from_top as f32 > height_pixels {
        return None;
    }

    let noise_0_to_1 = (noise_m1_to_1 + 1.0) / 2.0;
    let color = interpolate(noise_0_to_1, CLOUD_COLOR_DARK, CLOUD_COLOR_BRIGHT);

    // 0-1, higher is closer to the edge of the cloud
    let fraction_of_cloud_height = pixel_y_from_top as f32 / height_pixels;

    let opaque_fraction = 1.0 - CLOUD_TRANSPARENT_FRACTION;

    if fraction_of_cloud_height < opaque_fraction {
        // We're in the not-transparent part of the cloud
        return Some(color);
    }

    // 0-1, higher is more transparent
    let transparent_factor = (fraction_of_cloud_height - opaque_fraction) / (1.0 - opaque_fraction);
    return Some(interpolate(transparent_factor as f64, &color, BG_COLOR_RGB));
}

/// Turns `[3, 1, 2]` into `[1, 2, 3, 3, 2, 1]`
fn mirror_sort(cpu_loads: &Vec<CpuLoad>) -> Vec<CpuLoad> {
    let mut result = cpu_loads.clone();

    // Sort criteria is same as in `LoadViz::update_currently_displayed_loads()`
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for i in (0..cpu_loads.len()).rev() {
        result.push(result[i]);
    }

    return result;
}

fn interpolate(factor_0_to_1: f64, color1: &[u8; 3], color2: &[u8; 3]) -> [u8; 3] {
    let factor_0_to_1 = factor_0_to_1.clamp(0.0, 1.0);

    let mut result = [0; 3];

    for i in 0..3 {
        result[i] =
            (color1[i] as f64 * (1.0 - factor_0_to_1) + color2[i] as f64 * factor_0_to_1) as u8;
    }

    return result;
}

#[cfg(test)]
mod tests {
    use crate::cpuload::CpuLoad;

    use super::{mirror_sort, Renderer};

    #[test]
    fn test_interpolate() {
        let black: [u8; 3] = [0x00, 0x00, 0x00];
        let white: [u8; 3] = [0xff, 0xff, 0xff];

        assert_eq!(black, super::interpolate(0.0, &black, &white));
        assert_eq!(white, super::interpolate(1.0, &black, &white));
    }

    #[test]
    /// Test rendering an empty list of loads. The point is just that we
    /// shouldn't crash.
    fn test_render_nothing() {
        let width = 10;
        let height = 10;
        let mut pixels = vec![0; width * height * 3];
        let renderer: Renderer = Default::default();

        renderer.render_image(&Vec::new(), width, height, &mut pixels);
    }

    #[test]
    fn test_mirror_sort_empty() {
        assert_eq!(0, mirror_sort(&Vec::new()).len());
    }

    #[test]
    fn test_mirror_sort_same() {
        let mirror_sorted = mirror_sort(&vec![
            // This one is identical to...
            CpuLoad {
                user_0_to_1: 0.1,
                system_0_to_1: 0.2,
            },
            // ...this one.
            CpuLoad {
                user_0_to_1: 0.1,
                system_0_to_1: 0.2,
            },
        ]);

        assert_eq!(
            mirror_sorted,
            vec![
                CpuLoad {
                    user_0_to_1: 0.1,
                    system_0_to_1: 0.2,
                },
                CpuLoad {
                    user_0_to_1: 0.1,
                    system_0_to_1: 0.2,
                },
                CpuLoad {
                    user_0_to_1: 0.1,
                    system_0_to_1: 0.2,
                },
                CpuLoad {
                    user_0_to_1: 0.1,
                    system_0_to_1: 0.2,
                },
            ]
        );
    }
}
