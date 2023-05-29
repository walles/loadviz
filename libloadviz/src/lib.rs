#![allow(clippy::needless_return)]

// FIXME: Remove this function
#[no_mangle]
pub extern "C" fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[no_mangle]
pub extern "C" fn get_image(width: usize, height: usize) -> *const u8 {
    let mut image: Vec<u8> = vec![0; width * height * 3];

    // Set every third byte to 255 to make the image red
    for i in (0..image.len()).step_by(3) {
        image[i] = 255;
    }

    // FIXME: Make sure this memory isn't just leaked!

    // FIXME: How do we know this doesn't just get free()d by Rust before we
    // even return it?
    let boxed_image = image.into_boxed_slice();
    return boxed_image.as_ptr();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
