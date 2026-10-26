use bracket_noise::prelude::{FastNoise, NoiseType};

use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];

pub struct Renderer {
    /// Perlin noise for the flames
    noise: FastNoise,

    /// Value noise for the clouds: cheap to sample, and the clouds are soft
    /// enough to carry it.
    cloud_noise: FastNoise,
}

impl Default for Renderer {
    fn default() -> Self {
        let mut noise = FastNoise::new();
        noise.set_noise_type(NoiseType::Perlin);

        let mut cloud_noise = FastNoise::new();
        cloud_noise.set_noise_type(NoiseType::Value);

        return Self {
            noise,
            cloud_noise,
        };
    }
}

/// What stays the same for every pixel of one rendered frame
struct Frame<'a> {
    viz_loads: &'a [CpuLoad],
    dt_seconds: f32,
    width: usize,
    height: usize,
}

mod cloud;
mod flame;

impl Renderer {
    /// Don't call this! It's public for benchmarking purposes only.
    ///
    /// You should call `LoadViz::render_image()` instead.
    pub fn render_image(
        &self,
        currently_displayed_loads: &[CpuLoad],
        width: usize,
        height: usize,
        dt_seconds: f32,
        pixels: &mut [u8],
    ) {
        if currently_displayed_loads.is_empty() {
            // FIXME: Draw something nice?
            return;
        }

        // Too high values make the flames look like they have JPEG artifacts.
        //
        // Experiments showed that 14 days was really bad. 7 days was not bad.
        // So let's cap it at somewhere around 70% of one day so that the cut
        // comes at different times every day.
        let dt_seconds = dt_seconds % (3600.0 * 17.3);

        let viz_loads = mirror_sort(currently_displayed_loads);

        // A pixel column's flame can only be as tall as the loads the
        // horizontal distortion can reach sideways into it. Precomputing that
        // per-column ceiling lets `get_flame_pixel()` skip the noise maths for
        // every pixel above it.
        let column_ceiling_load_0_to_1 = max_reachable_user_load_by_column(&viz_loads, width, height);

        let frame = Frame {
            viz_loads: &viz_loads,
            dt_seconds,
            width,
            height,
        };

        #[allow(clippy::needless_range_loop)]
        for pixel_x in 0..width {
            for pixel_y_from_top in 0..height {
                let pixel_y_from_bottom = height - 1 - pixel_y_from_top;

                let color = if let Some(flame_color) = self.get_flame_pixel(
                    &frame,
                    column_ceiling_load_0_to_1[pixel_x],
                    pixel_x,
                    pixel_y_from_bottom,
                ) {
                    flame_color
                } else if let Some(cloud_color) = self.get_cloud_pixel(&frame, pixel_x, pixel_y_from_top) {
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

fn get_load(viz_loads: &[CpuLoad], x_fraction_0_to_1: f32) -> CpuLoad {
    let float_part_index = load_index(x_fraction_0_to_1, viz_loads.len());
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

/// For each pixel column, the tallest user load whose flame can reach that
/// column.
///
/// This is a safe over-estimate: culling a pixel against it never removes
/// anything `get_flame_pixel()` would have drawn.
fn max_reachable_user_load_by_column(viz_loads: &[CpuLoad], width: usize, height: usize) -> Vec<f32> {
    // A column can show a flame sourced from any load within this reach.
    let reach_pixels = flame::distortion_pixel_radius(width, height).ceil() as usize;

    let mut result = Vec::with_capacity(width);
    for pixel_x in 0..width {
        // The distortion samples the load somewhere in this pixel range, which
        // `get_load()` maps onto a range of load buckets. Take the peak over the
        // raw buckets: a load peak that falls between two columns must still
        // raise the ceiling, or the guard could cull a pixel the flame fills.
        let leftmost_x = pixel_x as f32 - reach_pixels as f32;
        let rightmost_x = pixel_x as f32 + reach_pixels as f32;
        let first_bucket = load_bucket(leftmost_x, width, viz_loads.len()).floor() as usize;
        let last_bucket = load_bucket(rightmost_x, width, viz_loads.len()).ceil() as usize;

        let max_reachable_load_0_to_1 = viz_loads[first_bucket..=last_bucket]
            .iter()
            .map(|load| load.user_0_to_1)
            .fold(0.0f32, f32::max);

        result.push(max_reachable_load_0_to_1);
    }

    return result;
}

/// Where pixel column `pixel_x` falls among `load_count` load buckets spread
/// across a `width`-wide image, as a fractional index. Flooring and ceiling it
/// gives the two buckets it lies between — both always valid in `0..load_count`,
/// even for off-image columns.
fn load_bucket(pixel_x: f32, width: usize, load_count: usize) -> f32 {
    return load_index(pixel_to_fraction(pixel_x, width), load_count);
}

/// The (fractional) load-array index a column fraction maps to, clamped to a
/// valid index in `0..load_count`. Flooring and ceiling it gives the two
/// buckets it lies between.
fn load_index(x_fraction_0_to_1: f32, load_count: usize) -> f32 {
    return (load_count as f32 * x_fraction_0_to_1 - 0.5).clamp(0.0, load_count as f32 - 1.0);
}

/// Turns `[3, 1, 2]` into `[1, 2, 3, 3, 2, 1]`
fn mirror_sort(cpu_loads: &[CpuLoad]) -> Vec<CpuLoad> {
    let mut result = cpu_loads.to_vec();

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

    /// Details should be sized relative to the image height, so that a square
    /// icon gets the same detail size as a 2:1 one.
    #[test]
    fn test_details_follow_height() {
        let load = CpuLoad {
            user_0_to_1: 0.6,
            system_0_to_1: 0.3,
        };
        let loads = vec![load; 4];
        let renderer: Renderer = Default::default();

        let mut square = vec![0u8; 32 * 32 * 3];
        renderer.render_image(&loads, 32, 32, 1.5, &mut square);

        let mut wide = vec![0u8; 64 * 32 * 3];
        renderer.render_image(&loads, 64, 32, 1.5, &mut wide);

        for y in 0..32 {
            let square_row = &square[y * 32 * 3..(y + 1) * 32 * 3];
            let wide_row_left_half = &wide[y * 64 * 3..y * 64 * 3 + 32 * 3];
            assert_eq!(square_row, wide_row_left_half, "Row {y} differs");
        }
    }

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
        let mirror_sorted = mirror_sort(&[
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
