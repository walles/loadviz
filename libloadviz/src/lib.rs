#![allow(clippy::needless_return)]

pub struct LoadViz {
    width: usize,
    height: usize,

    // FIXME: Remove this, used only for early development
    value: u8,

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

        // Change color every time so we can see when it updates
        self.value += 57;
        for i in 0..self.pixels.len() {
            self.pixels[i] = self.value;
        }

        return &self.pixels[0]
    }
}

#[no_mangle]
pub extern "C" fn new_loadviz() -> *mut LoadViz {
    return opaque_pointer::raw(LoadViz {
        width: 0,
        height: 0,
        pixels: vec![0],
        value: 0,
    });
}

/// # Safety
///
/// This function is unsafe because it dereferences the incoming `loadviz`
/// pointer. But as long as you get that from [`new_loadviz()`](new_loadviz) you
/// should be fine.
#[no_mangle]
pub unsafe extern "C" fn get_image(loadviz: *mut LoadViz, width: usize, height: usize) -> *const u8 {
    let loadviz = unsafe { opaque_pointer::mut_object(loadviz) };
    let loadviz = loadviz.unwrap();
    return loadviz.get_image(width, height);
}
