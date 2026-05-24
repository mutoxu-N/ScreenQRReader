// スクリーンショット撮影とトリミング

use base64::{engine::general_purpose, Engine as _};
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::wry::dpi::{PhysicalPosition, PhysicalSize};
use dioxus::desktop::{Config, DesktopContext, WindowBuilder};
use dioxus::prelude::*;
use image::ImageFormat;
use std::cell::RefCell;
use std::io::Cursor;
use xcap::Monitor;

use crate::components::overlay::{Overlay, OverlayProps};

// 二重起動防止

pub struct MonitorCapture {
    pub base64: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

// ウィンドウコンテキストを保持するスレッドローカル変数
thread_local! {
    static OVERLAY_WINDOWS: RefCell<Vec<DesktopContext>> = RefCell::new(Vec::new());
}

pub fn register_overlay_window(window: DesktopContext) {
    OVERLAY_WINDOWS.with(|windows| {
        windows.borrow_mut().push(window);
    });
}

pub fn close_all_overlays() {
    OVERLAY_WINDOWS.with(|windows| {
        for w in windows.borrow_mut().drain(..) {
            w.close();
        }
    });
}

pub fn capture_all_monitors() -> Vec<MonitorCapture> {
    let monitors = match Monitor::all() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("モニターの取得に失敗しました: {}", e);
            return Vec::new();
        }
    };

    let mut captures = Vec::new();

    for m in monitors {
        let Ok(x) = m.x() else {
            continue;
        };
        let Ok(y) = m.y() else {
            continue;
        };
        let Ok(w) = m.width() else {
            continue;
        };
        let Ok(h) = m.height() else {
            continue;
        };

        if let Ok(image) = m.capture_image() {
            let mut cursor = Cursor::new(Vec::new());

            // JPEGでエンコード
            let dyn_img = image::DynamicImage::ImageRgba8(image);
            if dyn_img
                .into_rgb8()
                .write_to(&mut cursor, ImageFormat::Jpeg)
                .is_ok()
            {
                let base64_str = format!(
                    "data:image/jpeg;base64,{}",
                    general_purpose::STANDARD.encode(cursor.into_inner())
                );

                captures.push(MonitorCapture {
                    base64: base64_str,
                    x: x,
                    y: y,
                    width: w,
                    height: h,
                })
            }
        }
    }

    captures
}

pub fn spawn_overlay_windows(captures: Vec<MonitorCapture>) {
    // 以前のウィンドウリストが残っていたらクリア
    OVERLAY_WINDOWS.with(|windows| {
        windows.borrow_mut().clear();
    });

    for m in captures {
        let img = m.base64.clone();

        let _ = dioxus::desktop::window().new_window(
            VirtualDom::new_with_props(Overlay, OverlayProps { image_data: img }),
            Config::new().with_window(
                WindowBuilder::new()
                    .with_visible(true)
                    .with_decorations(false)
                    .with_resizable(false)
                    .with_undecorated_shadow(false)
                    .with_skip_taskbar(true)
                    .with_transparent(true)
                    .with_always_on_top(true)
                    .with_title("Screen QR Reader")
                    .with_position(PhysicalPosition { x: m.x, y: m.y })
                    .with_inner_size(PhysicalSize {
                        width: m.width,
                        height: m.height,
                    }),
            ),
        );
    }
}
