#![allow(clippy::needless_return)]

use std::time::Instant;

use physics::update_currently_displayed_loads;
use system_load::get_load_counters;

pub mod cpuload;
mod load_reader;
mod physics;

pub mod system_load;

// Public for benchmarking purposes only
pub mod renderer;

pub struct LoadViz {
    width: usize,
    height: usize,

    /// Size: 3 * width * height. Format: RGBRGBRGB...
    pixels: Vec<u8>,

    /// What we're currently displaying. This will constantly be animated
    /// towards the current system load.
    currently_displayed_loads: Vec<cpuload::CpuLoad>,
    currently_displayed_loads_updated: std::time::Instant,

    /// When this object was created
    t0: std::time::Instant,

    load_reader: load_reader::LoadReader,

    renderer: renderer::Renderer,
}

impl LoadViz {
    pub fn new() -> LoadViz {
        todo!();
    }

    /// Render the current system load into an image.
    ///
    /// Returns `3 * width * height` bytes. Format: RGBRGBRGB..., starting at
    /// the top left corner and going one row at a time.
    pub fn get_image(&mut self, _width: usize, _height: usize) -> &[u8] {
        todo!();
    }
}

impl Default for LoadViz {
    fn default() -> Self {
        return LoadViz::new();
    }
}

#[no_mangle]
pub extern "C" fn new_loadviz() -> *mut LoadViz {
    return opaque_pointer::raw(LoadViz::new());
}

/// # Safety
///
/// This function is unsafe because it dereferences the incoming `loadviz`
/// pointer. But as long as you get that from [`new_loadviz()`](new_loadviz) you
/// should be fine.
#[no_mangle]
pub unsafe extern "C" fn get_image(
    loadviz: *mut LoadViz,
    width: usize,
    height: usize,
) -> *const u8 {
    let loadviz = unsafe { opaque_pointer::mut_object(loadviz) };
    let loadviz = loadviz.unwrap();
    return loadviz.get_image(width, height).as_ptr();
}

#[cfg(test)]
mod tests {
    use super::LoadViz;

    #[test]
    fn test_get_image_size() {
        let mut loadviz = LoadViz::new();
        assert_eq!(3 * 10 * 7, loadviz.get_image(10, 7).len());
        assert_eq!(3 * 4 * 5, loadviz.get_image(4, 5).len());
    }
}
