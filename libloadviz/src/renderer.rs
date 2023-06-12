use crate::{cpuload::CpuLoad, LoadViz};

static BG_COLOR_RGB: &[u8] = &[0x30, 0x30, 0x90];
static BG_COLOR_RGB_DARK: &[u8] = &[0x20, 0x20, 0x60];
static USER_LOAD_COLOR_RGB: &[u8] = &[0x00, 0xff, 0x00];
static USER_LOAD_COLOR_RGB_DARK: &[u8] = &[0x00, 0xb0, 0x00];
static SYSTEM_LOAD_COLOR_RGB: &[u8] = &[0xff, 0x00, 0x00];
static SYSTEM_LOAD_COLOR_RGB_DARK: &[u8] = &[0xb0, 0x00, 0x00];

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

    // Make square boxes
    let divider_distance = (width as f32 / viz_loads.len() as f32) as usize;

    for i in (0..pixels.len()).step_by(3) {
        let x = (i / 3) % width;
        let y = height - (i / 3) / width - 1;

        let dark = y % divider_distance == 0 || x % divider_distance == 0;

        let cpu_load = &viz_loads[(x * viz_loads.len()) / width];

        let y_height = y as f32 / height as f32;
        let user_plus_system_height = cpu_load.user_0_to_1 + cpu_load.system_0_to_1;
        let color = if y_height > user_plus_system_height {
            if dark {
                BG_COLOR_RGB_DARK
            } else {
                BG_COLOR_RGB
            }
        } else if y_height > cpu_load.system_0_to_1 {
            if dark {
                USER_LOAD_COLOR_RGB_DARK
            } else {
                USER_LOAD_COLOR_RGB
            }
        } else if dark {
            SYSTEM_LOAD_COLOR_RGB_DARK
        } else {
            SYSTEM_LOAD_COLOR_RGB
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

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use crate::cpuload::CpuLoad;

    use super::mirror_sort;

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
