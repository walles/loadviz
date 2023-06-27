#![allow(clippy::needless_return)]

use std::time::Instant;

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
    pub(crate) fn get_image(&mut self, width: usize, height: usize) -> *const u8 {
        if width != self.width || height != self.height {
            self.width = width;
            self.height = height;
            self.pixels = vec![0; width * height * 3];
        }

        if self.load_reader.get_loads().is_empty() {
            // FIXME: Draw something nice?
            return &self.pixels[0];
        }

        self.update_currently_displayed_loads();

        self.renderer.render_image(
            &self.currently_displayed_loads,
            self.width,
            self.height,
            self.t0.elapsed().as_secs_f64(),
            &mut self.pixels,
        );

        return &self.pixels[0];
    }
}

#[no_mangle]
pub extern "C" fn new_loadviz() -> *mut LoadViz {
    return opaque_pointer::raw(LoadViz {
        width: 0,
        height: 0,
        pixels: vec![0],
        currently_displayed_loads: Vec::new(),
        currently_displayed_loads_updated: std::time::Instant::now(),
        t0: Instant::now(),
        load_reader: load_reader::LoadReader::new(get_load_counters),
        renderer: Default::default(),
    });
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
    return loadviz.get_image(width, height);
}
