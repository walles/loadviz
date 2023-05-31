#![allow(clippy::needless_return)]

pub struct LoadViz {
    width: usize,
    height: usize,

    // Size: 3* width * height. Format: RGBRGBRGB...
    pixels: Vec<u8>,
}

impl LoadViz {
    pub fn get_image(&mut self, width: usize, height: usize) -> *const u8 {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.pixels = vec![0; width * height * 3];
        }

        // Set every third byte to 255 to make the image red
        for i in (0..self.pixels.len()).step_by(3) {
            self.pixels[i] = 255;
        }

        return &self.pixels[0]
    }
}

#[no_mangle]
pub extern "C" fn loadviz_new() -> *mut LoadViz {
    return opaque_pointer::raw(LoadViz {
        width: 0,
        height: 0,
        pixels: vec![0],
    });
}

/// # Safety
///
/// This function is unsafe because it dereferences the incoming load_viz
/// pointer. But as long as you get that from `loadviz_new()` you should be
/// fine.
#[no_mangle]
pub unsafe extern "C" fn get_image(load_viz: *mut LoadViz, width: usize, height: usize) -> *const u8 {
    let load_viz = unsafe { opaque_pointer::mut_object(load_viz) };
    let load_viz = load_viz.unwrap();
    return load_viz.get_image(width, height);
}
