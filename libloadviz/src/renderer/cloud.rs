use crate::cpuload::CpuLoad;

use super::{get_load, interpolate, Renderer, BG_COLOR_RGB};

static CLOUD_COLOR_DARK: &[u8; 3] = &[0x88, 0x88, 0x88];
static CLOUD_COLOR_BRIGHT: &[u8; 3] = &[0xff, 0xff, 0xff];

/// How much of the cloud should fade towards transparent?
///
/// This is a fraction of the height of the whole image, not a fraction of the
/// height of the cloud.
///
/// Lower values make the cloud edge sharper.
static CLOUD_TRANSPARENT_FRACTION: f32 = 0.4;

impl Renderer {
    pub(super) fn get_cloud_pixel(
        &self,
        viz_loads: &Vec<CpuLoad>,
        dt_seconds: f32,
        pixel_x: usize,
        pixel_y_from_top: usize,
        width: usize,
        height: usize,
    ) -> Option<[u8; 3]> {
        // Higher number = more details.
        let detail = 5.0 / width as f32;

        // Higher speed number = faster cloud turbulence.
        let speed = 0.3;

        let x_fraction_0_to_1 = pixel_x as f32 / (width as f32 - 1.0);
        let cpu_load = get_load(viz_loads, x_fraction_0_to_1);

        // Compute the sysload height for this load
        let cloud_height_pixels = cpu_load.system_0_to_1 * height as f32;
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

        let transparency_height_pixels = CLOUD_TRANSPARENT_FRACTION * height as f32;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_load_no_cloud() {
        let viz_loads = vec![CpuLoad {
            user_0_to_1: 0.0,
            system_0_to_1: 0.0,
        }];
        let renderer: Renderer = Default::default();
        let pixel = renderer.get_cloud_pixel(&viz_loads, 0.0, 0, 0, 1, 1);
        assert_eq!(pixel, None);
    }
}
