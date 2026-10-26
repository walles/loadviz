//! Compare rendered images against reference images, so that renderer changes
//! can't silently change what existing users see.
//!
//! To regenerate the reference images after an intentional change:
//!
//! ```sh
//! LOADVIZ_UPDATE_REFERENCES=1 cargo test --test rendering_reference
//! ```

#![allow(clippy::needless_return)]

use std::path::PathBuf;

use libloadviz::{cpuload::CpuLoad, renderer::Renderer};

/// Floating point results may differ slightly between CPU architectures
const MAX_CHANNEL_DIFF: u8 = 2;

fn render(renderer: &Renderer, width: usize, height: usize) -> Vec<u8> {
    let loads = vec![
        CpuLoad {
            user_0_to_1: 0.8,
            system_0_to_1: 0.1,
        },
        CpuLoad {
            user_0_to_1: 0.4,
            system_0_to_1: 0.4,
        },
        CpuLoad {
            user_0_to_1: 0.1,
            system_0_to_1: 0.6,
        },
    ];

    let mut pixels = vec![0u8; width * height * 3];
    renderer.render_image(&loads, width, height, 1.5, &mut pixels);
    return pixels;
}

fn assert_matches_reference(renderer: &Renderer, name: &str, width: usize, height: usize) {
    let pixels = render(renderer, width, height);

    let path: PathBuf = [
        env!("CARGO_MANIFEST_DIR"),
        "tests",
        "reference",
        &format!("{name}-{width}x{height}.rgb"),
    ]
    .iter()
    .collect();

    if std::env::var("LOADVIZ_UPDATE_REFERENCES").is_ok() {
        std::fs::write(&path, &pixels).unwrap();
        return;
    }

    let reference = std::fs::read(&path).unwrap();
    assert_eq!(reference.len(), pixels.len());
    for (i, (actual, expected)) in pixels.iter().zip(reference.iter()).enumerate() {
        let pixel = i / 3;
        assert!(
            actual.abs_diff(*expected) <= MAX_CHANNEL_DIFF,
            "{name} {width}x{height}: pixel ({}, {}) channel {} is {actual}, expected {expected}",
            pixel % width,
            pixel / width,
            i % 3,
        );
    }
}

/// The macOS menu bar icon on a 2x Retina display
#[test]
fn test_macos_retina() {
    assert_matches_reference(&Default::default(), "macos", 88, 44);
}

/// The macOS menu bar icon on a 1x display
#[test]
fn test_macos_1x() {
    assert_matches_reference(&Default::default(), "macos", 44, 22);
}

/// `screenshot.webp`, from the `animated-webp` binary
#[test]
fn test_screenshot() {
    assert_matches_reference(&Renderer::with_width_based_details(), "screenshot", 100, 75);
}

/// A smaller version of the 1024x1024 app icon source image from the
/// `still-webp` binary. Same aspect ratio, so it catches scaling changes.
#[test]
fn test_app_icon() {
    assert_matches_reference(&Renderer::with_width_based_details(), "app-icon", 128, 128);
}
