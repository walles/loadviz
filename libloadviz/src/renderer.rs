static AIR_COLOR_RGB: &[u8] = &[0xc0, 0xc0, 0xff];
static LOAD_COLOR_RGB: &[u8] = &[0x00, 0xc0, 0x00];

pub(crate) fn render_image(heights_0_to_1: &Vec<f32>, width: usize, height: usize, pixels: &mut Vec<u8>) {
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