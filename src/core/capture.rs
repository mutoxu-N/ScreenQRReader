// スクリーンショット撮影

use image::RgbaImage;
use std::sync::atomic::{AtomicBool, Ordering};
use xcap::Monitor;

// 二重起動防止フラグ
static IS_ACTIVE: AtomicBool = AtomicBool::new(false);

pub fn try_start_capture() -> bool {
    IS_ACTIVE
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
}

pub fn finish_capture() {
    IS_ACTIVE.store(false, Ordering::SeqCst);
}

pub struct MonitorCapture {
    pub image: RgbaImage,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
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
        let Ok(x) = m.x() else { continue; };
        let Ok(y) = m.y() else { continue; };
        let Ok(w) = m.width() else { continue; };
        let Ok(h) = m.height() else { continue; };

        if let Ok(image) = m.capture_image() {
            captures.push(MonitorCapture {
                image,
                x,
                y,
                width: w,
                height: h,
            });
        }
    }

    captures
}
