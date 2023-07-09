use bracket_noise::prelude::FastNoise;

use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];

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
mod flame;

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

                let color = if let Some(flame_color) = self.get_flame_pixel(
                    &viz_loads,
                    dt_seconds,
                    pixel_x,
                    pixel_y_from_bottom,
                    width,
                    height,
                ) {
                    flame_color
                } else if let Some(cloud_color) = self.get_cloud_pixel(
                    &viz_loads,
                    dt_seconds,
                    pixel_x,
                    pixel_y_from_top,
                    width,
                    height,
                ) {
                    cloud_color
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
