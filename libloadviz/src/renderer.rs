use bracket_noise::prelude::FastNoise;

use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];

// Blackbody RGB values from: http://www.vendian.org/mncharity/dir3/blackbody/
static USER_LOAD_COLOR_RGB_WARMER: &[u8; 3] = &[0xff, 0xb4, 0x6b]; // 3000K
static USER_LOAD_COLOR_RGB_COOLER: &[u8; 3] = &[0xff, 0x38, 0x00]; // 1000K

pub struct Renderer {
    noise: FastNoise,
}

impl Default for Renderer {
    fn default() -> Self {
        return Self {
            noise: FastNoise::new(),
        };
    }
}

mod cloud;

impl Renderer {
    /// Don't call this! It's public for benchmarking purposes only.
    ///
    /// You should call `LoadViz::render_image()` instead.
    pub fn render_image(
        &self,
        currently_displayed_loads: &Vec<CpuLoad>,
        width: usize,
        height: usize,
        dt_seconds: f32,
        pixels: &mut [u8],
    ) {
        if currently_displayed_loads.is_empty() {
            // FIXME: Draw something nice?
            return;
        }
        let viz_loads = mirror_sort(currently_displayed_loads);

        for pixel_x in 0..width {
            for pixel_y_from_top in 0..height {
                let pixel_y_from_bottom = height - 1 - pixel_y_from_top;

                let color = if let Some(cloud_color) = self.get_cloud_pixel(
                    &viz_loads,
                    dt_seconds,
                    pixel_x,
                    pixel_y_from_top,
                    width,
                    height,
                ) {
                    cloud_color
                } else if let Some(flame_color) = self.get_flame_pixel(
                    &viz_loads,
                    dt_seconds,
                    pixel_x,
                    pixel_y_from_bottom,
                    width,
                    height,
                ) {
                    flame_color
                } else {
                    *BG_COLOR_RGB
                };

                let i = 3 * (pixel_y_from_top * width + pixel_x);
                pixels[i] = color[0];
                pixels[i + 1] = color[1];
                pixels[i + 2] = color[2];
            }
        }
    }

    fn get_flame_pixel(
        &self,
        viz_loads: &Vec<CpuLoad>,
        dt_seconds: f32,
        pixel_x: usize,
        pixel_y_from_bottom: usize,
        width: usize,
        height: usize,
    ) -> Option<[u8; 3]> {
        // This number determines how uneven the edge of the fire is. Also, it
        // decides how much warping happens to the internal base image.
        let distortion_detail = 7.0 / width as f32;

        // This number decides how warped the internal base image is. Try
        // setting distortion_detail ^ to almost zero to see the effect of
        // changing this number.
        let internal_detail = 6.0 / width as f32;

        // What fraction of the inside of the fire fades towards transparent?
        let transparent_internal_0_to_1 = 0.3;

        let distortion_pixel_radius = width.min(height) as f32 / 10.0;

        // Check whether we should even try to do flames maths. This improves
        // our idle-system benchmark by 63%.
        let highest_load_0_to_1 = viz_loads
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
        let noise1_m1_to_1 = self.noise.get_noise3d(
            distortion_detail * pixel_x as f32,
            distortion_detail * pixel_y_from_bottom as f32,
            dt_seconds,
        );

        // Pick the load to show
        let dx_pixels = noise1_m1_to_1 * distortion_pixel_radius;
        let distorted_pixel_x = pixel_x as f32 + dx_pixels;
        let x_fraction_0_to_1 = pixel_to_fraction(distorted_pixel_x, width);
        let cpu_load = get_load(viz_loads, x_fraction_0_to_1);

        let highest_possible_flame_height_pixels =
            cpu_load.user_0_to_1 * height as f32 + distortion_pixel_radius;
        if pixel_y_from_bottom as f32 > highest_possible_flame_height_pixels {
            // We're above the flames at this particular column, no need for any
            // more (costly) noise maths.
            //
            // This check improves our busy benchmark by 5%.
            return None;
        }

        let noise2_m1_to_1 = self.noise.get_noise3d(
            distortion_detail * pixel_x as f32,
            distortion_detail * pixel_y_from_bottom as f32,
            -dt_seconds - 1.0,
        );

        // Figure out how to color the current pixel
        let dy_pixels = noise2_m1_to_1 * distortion_pixel_radius;
        let distorted_pixel_y = pixel_y_from_bottom as f32 + dy_pixels;
        let y_from_bottom_0_to_1 = pixel_to_fraction(distorted_pixel_y, height);
        if y_from_bottom_0_to_1 > cpu_load.user_0_to_1 {
            return None;
        }

        // Get a 0-1 noise value for this coordinate, that scrolls up with time
        let temperature_0_to_1 = (self.noise.get_noise(
            internal_detail * distorted_pixel_x,
            internal_detail * distorted_pixel_y - dt_seconds * 2.0,
        ) + 1.0)
            / 2.0;

        // Make the fire cooler the closer the top of the flame we get
        let bottom_cooling_layer_thickness_0_to_1 = 0.2;
        let cooling_factor = if y_from_bottom_0_to_1 > bottom_cooling_layer_thickness_0_to_1 {
            // Cool based on the percentage of the flame height. This looks better in general.
            let fraction_of_current_height = y_from_bottom_0_to_1 / cpu_load.user_0_to_1;

            // "0.7" makes 100% load look like 100% flame height. Without that
            // factor, 100% load looked like maybe 80% flame height.
            1.0 - fraction_of_current_height * 0.7
        } else {
            // Cool based on a fraction of the image height. This looks better
            // for low CPU loads / flame heights.
            let distance_from_top_0_to_1 = cpu_load.user_0_to_1 - y_from_bottom_0_to_1;
            1.0 - ((bottom_cooling_layer_thickness_0_to_1 - distance_from_top_0_to_1)
                .clamp(0.0, 1.0)
                / bottom_cooling_layer_thickness_0_to_1)
        };
        let temperature_0_to_1 = temperature_0_to_1 * cooling_factor;

        // Colorize based on the noise value
        let color = if temperature_0_to_1 < transparent_internal_0_to_1 {
            interpolate(
                temperature_0_to_1 / transparent_internal_0_to_1,
                BG_COLOR_RGB,
                USER_LOAD_COLOR_RGB_COOLER,
            )
        } else {
            interpolate(
                (temperature_0_to_1 - transparent_internal_0_to_1)
                    / (1.0 - transparent_internal_0_to_1),
                USER_LOAD_COLOR_RGB_COOLER,
                USER_LOAD_COLOR_RGB_WARMER,
            )
        };

        return Some(color);
    }
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

fn interpolate(factor_0_to_1: f32, color1: &[u8; 3], color2: &[u8; 3]) -> [u8; 3] {
    let factor_0_to_1 = factor_0_to_1.clamp(0.0, 1.0);

    let mut result = [0; 3];

    for i in 0..3 {
        result[i] =
            (color1[i] as f32 * (1.0 - factor_0_to_1) + color2[i] as f32 * factor_0_to_1) as u8;
    }

    return result;
}

fn pixel_to_fraction(pixel: f32, maxpixel: usize) -> f32 {
    return (pixel + 1.0) / (maxpixel + 2) as f32;
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

        renderer.render_image(&Vec::new(), width, height, 42.0, &mut pixels);
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

    #[test]
    fn test_pixel_to_fraction() {
        // Fractions should be evenly spaced
        assert_eq!(0.25, pixel_to_fraction(0.0, 2));
        assert_eq!(0.50, pixel_to_fraction(1.0, 2));
        assert_eq!(0.75, pixel_to_fraction(2.0, 2));
    }
}
