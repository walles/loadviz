#![allow(clippy::needless_return)]

use ksni::blocking::TrayMethods;
use libloadviz::LoadViz;

/// Width and height of the icon in pixels. GNOME draws tray icons 16 logical
/// pixels high, which is 32 physical pixels on a 2x display.
const ICON_SIZE: usize = 32;

/// 10 frames per second, like on macOS
const FRAME_INTERVAL: std::time::Duration = std::time::Duration::from_millis(100);

struct LoadVizTray {
    icon: ksni::Icon,
}

impl ksni::Tray for LoadVizTray {
    // Open the menu on left click as well, like on macOS
    const MENU_ON_ACTIVATE: bool = true;

    fn id(&self) -> String {
        return "loadviz".into();
    }

    fn title(&self) -> String {
        return "LoadViz".into();
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        return vec![self.icon.clone()];
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        return vec![ksni::menu::StandardItem {
            label: "Quit".into(),
            activate: Box::new(|_| std::process::exit(0)),
            ..Default::default()
        }
        .into()];
    }
}

fn main() {
    let mut loadviz = LoadViz::new();

    let tray = LoadVizTray {
        icon: render_icon(&mut loadviz),
    };
    let handle = tray.spawn().expect("Registering the tray icon failed");

    loop {
        std::thread::sleep(FRAME_INTERVAL);

        let icon = render_icon(&mut loadviz);
        handle.update(|tray| tray.icon = icon);
    }
}

fn render_icon(loadviz: &mut LoadViz) -> ksni::Icon {
    let rgb = loadviz.get_image(ICON_SIZE, ICON_SIZE);
    return ksni::Icon {
        width: ICON_SIZE as i32,
        height: ICON_SIZE as i32,
        data: rgb_to_argb(rgb),
    };
}

/// Convert an RGB image from `libloadviz` into the ARGB32 format, network byte
/// order, that StatusNotifierItem icons use.
fn rgb_to_argb(rgb: &[u8]) -> Vec<u8> {
    let mut argb = Vec::with_capacity(rgb.len() / 3 * 4);
    for pixel in rgb.as_chunks::<3>().0 {
        argb.push(0xff);
        argb.extend_from_slice(pixel);
    }
    return argb;
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
