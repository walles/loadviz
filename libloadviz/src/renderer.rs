use crate::cpuload::CpuLoad;

static BG_COLOR_RGB: &[u8] = &[0x00, 0x00, 0x00];
static USER_LOAD_COLOR_RGB: &[u8] = &[0x00, 0xff, 0x00];
static SYSTEM_LOAD_COLOR_RGB: &[u8] = &[0xff, 0x00, 0x00];

/// `heights_0_to_1` is a list of heights between 0 and 1 in no particular order
pub(crate) fn render_image(
    cpu_loads: &Vec<CpuLoad>,
    width: usize,
    height: usize,
    pixels: &mut Vec<u8>,
) {
    if cpu_loads.is_empty() {
        // FIXME: Draw something nice?
        return;
    }

    let cpu_loads = mirror_sort(cpu_loads);

    // Make square boxes
    let divider_distance = (width as f32 / cpu_loads.len() as f32) as usize;

    for i in (0..pixels.len()).step_by(3) {
        let x = (i / 3) % width;
        let y = height - (i / 3) / width - 1;

        if y % divider_distance == 0 || x % divider_distance == 0 {
            pixels[i] = BG_COLOR_RGB[0];
            pixels[i + 1] = BG_COLOR_RGB[1];
            pixels[i + 2] = BG_COLOR_RGB[2];
            continue;
        }

        let cpu_load = &cpu_loads[(x * cpu_loads.len()) / width];

        let y_height = y as f32 / height as f32;
        let user_plus_system_height = cpu_load.user_0_to_1 + cpu_load.system_0_to_1;
        let color = if y_height > user_plus_system_height {
            BG_COLOR_RGB
        } else if y_height > cpu_load.system_0_to_1 {
            USER_LOAD_COLOR_RGB
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
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for i in (0..cpu_loads.len()).rev() {
        result.push(result[i]);
    }

    return result;
}

#[cfg(test)]
mod tests {
    use crate::cpuload::CpuLoad;

    use super::{mirror_sort, render_image};

    #[test]
    fn test_render_nothing() {
        // Test rendering an empty list of loads. The point is just that we
        // shouldn't crash.
        let empty_loads = vec![];

        let width = 10;
        let height = 10;
        let mut pixels = vec![0; width * height * 3];
        render_image(&empty_loads, width, height, &mut pixels);
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
