#![allow(clippy::needless_return)]

fn main() {
    todo!();
}

/// Convert an RGB image from `libloadviz` into the ARGB32 format, network byte
/// order, that StatusNotifierItem icons use.
fn rgb_to_argb(_rgb: &[u8]) -> Vec<u8> {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::rgb_to_argb;

    #[test]
    fn test_rgb_to_argb() {
        let rgb = [1, 2, 3, 4, 5, 6];
        assert_eq!(vec![0xff, 1, 2, 3, 0xff, 4, 5, 6], rgb_to_argb(&rgb));
    }

    #[test]
    fn test_rgb_to_argb_empty() {
        assert_eq!(Vec::<u8>::new(), rgb_to_argb(&[]));
    }
}
