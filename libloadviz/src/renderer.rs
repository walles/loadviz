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
static USER_LOAD_COLOR_RGB_WARMER: &[u8; 3] = &[0xff, 0xb4, 0x6b]; // 3000K

static CLOUD_COLOR_DARK: &[u8; 3] = &[0x88, 0x88, 0x88];
static CLOUD_COLOR_BRIGHT: &[u8; 3] = &[0xff, 0xff, 0xff];

/// How much of the cloud should fade towards transparent?
///
/// This is a fraction of the height of the whole image, not a fraction of the
/// height of the cloud.
///
/// Lower values make the cloud edge sharper.
static CLOUD_TRANSPARENT_FRACTION: f32 = 0.25;

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
        if currently_displayed_loads.is_empty() {
            // FIXME: Draw something nice?
            return;
        }
        let viz_loads = mirror_sort(currently_displayed_loads);

        let dt = self.t0.elapsed().as_secs_f64();
        for i in (0..pixels.len()).step_by(3) {
            let pixel_x = (i / 3) % width;
            let pixel_y_from_top = (i / 3) / width;
            let pixel_y_from_bottom = height - 1 - pixel_y_from_top;

            // Higher scale number = more details.
            let scale = 20.0 / width as f64;

            // NOTE: Experiments show that the Perlin output is -1 to 1
            let noise1_m1_to_1 =
                self.perlin
                    .get([scale * pixel_x as f64, scale * pixel_y_from_top as f64, dt]);
            let noise2_m1_to_1 = self.perlin.get([
                scale * pixel_x as f64,
                scale * pixel_y_from_top as f64,
                -dt - 1.0,
            ]);

            let color = if let Some(cloud_color) = get_cloud_pixel(
                &viz_loads,
                pixel_x,
                pixel_y_from_top,
                width,
                height,
                noise1_m1_to_1,
            ) {
                cloud_color
            } else if let Some(flame_color) = get_flame_pixel(
                &viz_loads,
                pixel_x,
                pixel_y_from_bottom,
                width,
                height,
                noise1_m1_to_1,
                noise2_m1_to_1,
            ) {
                flame_color
            } else {
                *BG_COLOR_RGB
            };

            pixels[i] = color[0];
            pixels[i + 1] = color[1];
            pixels[i + 2] = color[2];
        }
    }
}

fn get_flame_pixel(
    viz_loads: &Vec<CpuLoad>,
    pixel_x: usize,
    pixel_y_from_bottom: usize,
    width: usize,
    height: usize,
    noise1_m1_to_1: f64,
    noise2_m1_to_1: f64,
) -> Option<[u8; 3]> {
    let distortion_pixel_radius = 3.0;

    // Starting at this fraction of each flame pillar's height, the color will
    // start fading towards the background color.
    let fadeout_fraction = 0.8;

    // Pick the load to show
    let dx_pixels = noise1_m1_to_1 * distortion_pixel_radius;
    let distorted_x = pixel_x as f64 + dx_pixels;
    let x_fraction_0_to_1 = distorted_x as f32 / (width as f32 - 1.0);
    let cpu_load = get_load(viz_loads, x_fraction_0_to_1);

    // Figure out how to color the current pixel
    let dy_pixels = noise2_m1_to_1 * distortion_pixel_radius;
    let distorted_y = pixel_y_from_bottom as f64 + dy_pixels;
    let y_height_0_to_1 = distorted_y as f32 / height as f32;
    if y_height_0_to_1 > cpu_load.user_0_to_1 {
        return None;
    }

    // Get the base color
    let fraction = y_height_0_to_1 / cpu_load.user_0_to_1;
    let color = interpolate(
        fraction as f64,
        USER_LOAD_COLOR_RGB_WARMER,
        USER_LOAD_COLOR_RGB_COOLER,
    );

    if fraction < fadeout_fraction {
        // Too far from the tip, don't fade
        return Some(color);
    }

    // Fade out
    return Some(interpolate(
        ((fraction - fadeout_fraction) / (1.0 - fadeout_fraction)) as f64,
        &color,
        BG_COLOR_RGB,
    ));
}

fn get_cloud_pixel(
    viz_loads: &Vec<CpuLoad>,
    pixel_x: usize,
    pixel_y_from_top: usize,
    width: usize,
    height: usize,
    noise_m1_to_1: f64,
) -> Option<[u8; 3]> {
    let x_fraction_0_to_1 = pixel_x as f32 / (width as f32 - 1.0);
    let cpu_load = get_load(viz_loads, x_fraction_0_to_1);
    if cpu_load.system_0_to_1 < 0.01 {
        // Prevent a division by zero below
        return None;
    }

    // Compute the sysload height for this load
    let cloud_height_pixels = cpu_load.system_0_to_1 * height as f32;
    if pixel_y_from_top as f32 > cloud_height_pixels {
        return None;
    }

    let noise_0_to_1 = (noise_m1_to_1 + 1.0) / 2.0;
    let color = interpolate(noise_0_to_1, CLOUD_COLOR_DARK, CLOUD_COLOR_BRIGHT);

    let transparency_height_pixels = CLOUD_TRANSPARENT_FRACTION * height as f32;
    let opaque_height_pixels = cloud_height_pixels - transparency_height_pixels;
    if (pixel_y_from_top as f32) < opaque_height_pixels {
        return Some(color);
    }

    // 0-1, higher means more transparent
    let alpha =
        ((pixel_y_from_top as f32 - opaque_height_pixels) / transparency_height_pixels) as f64;
    return Some(interpolate(alpha, &color, BG_COLOR_RGB));
}

fn get_load(viz_loads: &Vec<CpuLoad>, x_fraction_0_to_1: f32) -> CpuLoad {
    let flen = viz_loads.len() as f32;
    let float_part_index = (flen * x_fraction_0_to_1 - 0.5).clamp(0.0, flen - 1.0);
    let i0 = float_part_index.floor() as usize;
    let i1 = float_part_index.ceil() as usize;
    if i0 == i1 {
        return viz_loads[i0];
    }

    let weight1 = 1.0 - (float_part_index - i0 as f32);
    let weight2 = 1.0 - (i1 as f32 - float_part_index);
    return CpuLoad {
        user_0_to_1: viz_loads[i0].user_0_to_1 * weight1 + viz_loads[i1].user_0_to_1 * weight2,
        system_0_to_1: viz_loads[i0].system_0_to_1 * weight1
            + viz_loads[i1].system_0_to_1 * weight2,
    };
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

    use super::*;

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

    /// Verify that with two loads:
    /// 0.00-0.25 Gives you load #1
    /// 0.25-0.75 Gives you points between load #1 and load #2
    /// 0.75-1.00 Gives you load #2
    #[test]
    fn test_get_load_2() {
        let example_loads = vec![
            CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            },
            CpuLoad {
                user_0_to_1: 1.0,
                system_0_to_1: 0.8,
            },
        ];
        assert_eq!(
            get_load(&example_loads, 0.0),
            CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            }
        );
        assert_eq!(
            get_load(&example_loads, 0.25),
            CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            }
        );
        assert_eq!(
            get_load(&example_loads, 0.5),
            CpuLoad {
                user_0_to_1: 0.5,
                system_0_to_1: 0.4,
            }
        );
        assert_eq!(
            get_load(&example_loads, 0.75),
            CpuLoad {
                user_0_to_1: 1.0,
                system_0_to_1: 0.8,
            }
        );
        assert_eq!(
            get_load(&example_loads, 1.0),
            CpuLoad {
                user_0_to_1: 1.0,
                system_0_to_1: 0.8,
            }
        );
    }
}
