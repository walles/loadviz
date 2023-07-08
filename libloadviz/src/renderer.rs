use bracket_noise::prelude::FastNoise;

use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];

static CLOUD_COLOR_DARK: &[u8; 3] = &[0x88, 0x88, 0x88];
static CLOUD_COLOR_BRIGHT: &[u8; 3] = &[0xff, 0xff, 0xff];

/// How much of the cloud should fade towards transparent?
///
/// This is a fraction of the height of the whole image, not a fraction of the
/// height of the cloud.
///
/// Lower values make the cloud edge sharper.
static CLOUD_TRANSPARENT_FRACTION: f32 = 0.4;

pub struct Renderer {
    noise: FastNoise,
    width: usize,
    height: usize,
}

mod fire;
mod load;

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        return Self {
            noise: FastNoise::new(),
            width,
            height,
        };
    }

    /// Don't call this! It's public for benchmarking purposes only.
    ///
    /// You should call `LoadViz::render_image()` instead.
    pub fn render_image(
        &self,
        currently_displayed_loads: &Vec<CpuLoad>,
        dt_seconds: f32,
        pixels: &mut [u8],
    ) {
        if currently_displayed_loads.is_empty() {
            // FIXME: Draw something nice?
            return;
        }
        let viz_loads = mirror_sort(currently_displayed_loads);

        for pixel_x in 0..self.width {
            for pixel_y_from_top in 0..self.height {
                let pixel_y_from_bottom = self.height - 1 - pixel_y_from_top;

                let color = if let Some(cloud_color) =
                    self.get_cloud_pixel(&viz_loads, dt_seconds, pixel_x, pixel_y_from_top)
                {
                    cloud_color
                } else if let Some(flame_color) =
                    self.get_flame_pixel(&viz_loads, dt_seconds, pixel_x, pixel_y_from_bottom)
                {
                    flame_color
                } else {
                    *BG_COLOR_RGB
                };

                let i = 3 * (pixel_y_from_top * self.width + pixel_x);
                pixels[i] = color[0];
                pixels[i + 1] = color[1];
                pixels[i + 2] = color[2];
            }
        }
    }

    fn get_cloud_pixel(
        &self,
        viz_loads: &Vec<CpuLoad>,
        dt_seconds: f32,
        pixel_x: usize,
        pixel_y_from_top: usize,
    ) -> Option<[u8; 3]> {
        // Higher number = more details.
        let detail = 5.0 / self.width as f32;

        // Higher speed number = faster cloud turbulence.
        let speed = 0.3;

        let cpu_load = self.get_load(viz_loads, pixel_x as f32);

        // Compute the sysload height for this load
        let cloud_height_pixels = cpu_load.system_0_to_1 * self.height as f32;
        if pixel_y_from_top as f32 > cloud_height_pixels {
            return None;
        }

        // Noise output is -1 to 1, deciphered from here:
        // https://github.com/amethyst/bracket-lib/blob/0d2d5e6a9a8e7c7ae3710cfef85be4cab0109a27/bracket-noise/examples/simplex_fractal.rs#L34-L39
        let noise_m1_to_1 = self.noise.get_noise3d(
            detail * pixel_x as f32,
            detail * pixel_y_from_top as f32,
            speed * dt_seconds,
        );

        let brightness_0_to_1 = (noise_m1_to_1 + 1.0) / 2.0;
        let color = interpolate(brightness_0_to_1, CLOUD_COLOR_DARK, CLOUD_COLOR_BRIGHT);

        let transparency_height_pixels = CLOUD_TRANSPARENT_FRACTION * self.height as f32;
        let opaque_height_pixels = cloud_height_pixels - transparency_height_pixels;
        if (pixel_y_from_top as f32) < opaque_height_pixels {
            // Cloud interior
            return Some(color);
        }

        // When we get here, we're closer to the edge of the cloud

        // 0-1, higher means more transparent
        let alpha = (pixel_y_from_top as f32 - opaque_height_pixels) / transparency_height_pixels;

        // Replace dark with transparent. Towards the edge of the cloud, we won't
        // see as many dark colors since the sun won't be blocked by thick cloud
        // parts.
        let color = interpolate(alpha * (1.0 - brightness_0_to_1), &color, BG_COLOR_RGB);

        return Some(interpolate(alpha, &color, BG_COLOR_RGB));
    }
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
        let renderer = Renderer::new(width, height);

        renderer.render_image(&Vec::new(), 42.0, &mut pixels);
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
