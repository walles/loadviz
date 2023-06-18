use noise::{NoiseFn, Perlin};

use crate::{cpuload::CpuLoad, LoadViz};

static BG_COLOR_RGB: &[u8; 3] = &[0x30, 0x30, 0x90];
static BG_COLOR_RGB_DARK: &[u8; 3] = &[0x18, 0x18, 0x48];
static USER_LOAD_COLOR_RGB: &[u8; 3] = &[0x00, 0xff, 0x00];
static USER_LOAD_COLOR_RGB_DARK: &[u8; 3] = &[0x00, 0x80, 0x00];
static SYSTEM_LOAD_COLOR_RGB: &[u8; 3] = &[0xff, 0x00, 0x00];
static SYSTEM_LOAD_COLOR_RGB_DARK: &[u8; 3] = &[0x80, 0x00, 0x00];

impl LoadViz {
    pub(crate) fn render_image(&mut self) {
        if self.load_reader.get_loads().is_empty() {
            // FIXME: Draw something nice?
            return;
        }

        self.update_currently_displayed_loads();

        render_image_raw(
            &self.currently_displayed_loads,
            self.width,
            self.height,
            &mut self.pixels,
        );
    }
}

/// Don't call this! It's public for benchmarking purposes only.
///
/// You should call `LoadViz::render_image()` instead.
pub fn render_image_raw(
    currently_displayed_loads: &Vec<CpuLoad>,
    width: usize,
    height: usize,
    pixels: &mut Vec<u8>,
) {
    let viz_loads = mirror_sort(currently_displayed_loads);

    // FIXME: Reuse the same Perlin instance for all calls to this function.
    let perlin = Perlin::new(Perlin::DEFAULT_SEED);

    for i in (0..pixels.len()).step_by(3) {
        let x = (i / 3) % width;
        let y = height - (i / 3) / width - 1;

        // FIXME: Pass a time dimension to the noise function. Higher number =
        // more details.
        let scale = 20.0 / width as f64;

        // NOTE: Experiments show that the output here is -1 to 1
        let number = perlin.get([scale * x as f64, scale * y as f64]);
        let number = (number + 1.0) / 2.0; // Normalize into 0 to 1

        let cpu_load = &viz_loads[(x * viz_loads.len()) / width];

        let y_height = y as f32 / height as f32;
        let idle_0_to_1 = 1.0 - (cpu_load.user_0_to_1 + cpu_load.system_0_to_1);
        let user_plus_idle_height = cpu_load.user_0_to_1 + idle_0_to_1;
        let color = if y_height > user_plus_idle_height {
            interpolate(number, SYSTEM_LOAD_COLOR_RGB_DARK, SYSTEM_LOAD_COLOR_RGB)
        } else if y_height > cpu_load.user_0_to_1 {
            interpolate(number, BG_COLOR_RGB_DARK, BG_COLOR_RGB)
        } else {
            interpolate(number, USER_LOAD_COLOR_RGB_DARK, USER_LOAD_COLOR_RGB)
        };

        pixels[i] = color[0];
        pixels[i + 1] = color[1];
        pixels[i + 2] = color[2];
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
    use std::time::Instant;

    use crate::cpuload::CpuLoad;

    use super::mirror_sort;

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
        let pixels = vec![0; width * height * 3];

        let mut loadviz = super::LoadViz {
            width,
            height,
            pixels,
            currently_displayed_loads: Vec::new(),
            currently_displayed_loads_updated: Instant::now(),

            // Create a load reader that always says there are no cores
            load_reader: crate::load_reader::LoadReader::new(std::vec::Vec::new),
        };
        loadviz.render_image();
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
