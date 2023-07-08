use crate::cpuload::CpuLoad;

use super::Renderer;

impl Renderer {
    pub(super) fn get_load(&self, viz_loads: &Vec<CpuLoad>, pixel_x: f32) -> CpuLoad {
        let flen = viz_loads.len() as f32;
        let x_fraction_0_to_1 = pixel_x / (self.width as f32 - 1.0);
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
}

#[cfg(test)]
mod tests {
    use crate::{cpuload::CpuLoad, renderer::Renderer};

    /// Verify that with two loads:
    /// 0.00-0.25 Gives you load #1
    /// 0.25-0.75 Gives you points between load #1 and load #2
    /// 0.75-1.00 Gives you load #2
    #[test]
    fn test_get_load() {
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

        let renderer = Renderer::new(101, 100);
        assert_eq!(
            renderer.get_load(&example_loads, 0.0),
            CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            }
        );
        assert_eq!(
            renderer.get_load(&example_loads, 25.0),
            CpuLoad {
                user_0_to_1: 0.0,
                system_0_to_1: 0.0,
            }
        );
        assert_eq!(
            renderer.get_load(&example_loads, 50.0),
            CpuLoad {
                user_0_to_1: 0.5,
                system_0_to_1: 0.4,
            }
        );
        assert_eq!(
            renderer.get_load(&example_loads, 75.0),
            CpuLoad {
                user_0_to_1: 1.0,
                system_0_to_1: 0.8,
            }
        );
        assert_eq!(
            renderer.get_load(&example_loads, 100.0),
            CpuLoad {
                user_0_to_1: 1.0,
                system_0_to_1: 0.8,
            }
        );
    }
}
