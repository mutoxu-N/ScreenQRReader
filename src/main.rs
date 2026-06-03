use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::wry::dpi::{LogicalPosition, LogicalSize};
use dioxus::desktop::{use_global_shortcut, Config, HotKeyState, WindowBuilder};
use dioxus::prelude::*;

mod components;
mod core;
mod state;

pub const MAIN_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // 画面の表示
    let cfg = Config::new().with_window(
        WindowBuilder::new()
            .with_title("Screen QR Reader")
            .with_visible(true)
            .with_decorations(false)
            .with_transparent(true)
            .with_skip_taskbar(true)
            .with_undecorated_shadow(false)
            .with_background_color((0, 0, 0, 0))
            .with_always_on_bottom(true)
            .with_resizable(false)
            .with_inner_size(LogicalSize::new(1, 1))
            .with_position(LogicalPosition::new(-10000, -10000)),
    );

    dioxus::LaunchBuilder::new().with_cfg(cfg).launch(App);
}

// キャプチャ処理を非同期で実行し、ウィンドウを生成する関数
fn trigger_capture() {
    if !core::capture::try_start_capture() {
        return;
    }

    spawn(async move {
        let captures = core::capture::capture_all_monitors();
        if captures.is_empty() {
            core::capture::cancel_capture();
        } else {
            core::capture::spawn_overlay_windows(captures);
        }
    });
}

#[component]
fn App() -> Element {
    core::tray::create_tray_menu();

    // ホットキーの登録
    if let Err(e) = use_global_shortcut("Ctrl+Shift+Q", move |state| {
        if state == HotKeyState::Pressed {
            trigger_capture();
        }
    }) {
        eprintln!("ホットキーの登録に失敗しました: {:?}", e);
    };

    rsx! {}
}
