static AIR_COLOR_RGB: &[u8] = &[0xc0, 0xc0, 0xff];
static LOAD_COLOR_RGB: &[u8] = &[0x00, 0xc0, 0x00];

/// `heights_0_to_1` is a list of heights between 0 and 1 in no particular order
pub(crate) fn render_image(heights_0_to_1: &Vec<f32>, width: usize, height: usize, pixels: &mut Vec<u8>) {
    let heights_0_to_1 = mirror_sort(heights_0_to_1);

    for i in (0..pixels.len()).step_by(3) {
        let x = (i / 3) % width;
        let y = height - (i / 3) / width - 1;

        let height_0_to_1 = heights_0_to_1[(x * heights_0_to_1.len()) / width ];

        let color = if y as f32 / height as f32 > height_0_to_1 {
            AIR_COLOR_RGB
        } else {
            LOAD_COLOR_RGB
        };

        pixels[i] = color[0];
        pixels[i + 1] = color[1];
        pixels[i + 2] = color[2];
    }
}

/// Turns `[3, 1, 2]` into `[1, 2, 3, 3, 2, 1]`
fn mirror_sort(heights_0_to_1: &Vec<f32>) -> Vec<f32> {
    let mut result = heights_0_to_1.clone();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap());

    for i in (0..heights_0_to_1.len()).rev() {
        result.push(result[i]);
    }

    return result
}
